//! Board-only firmware shell: the car AP, the UDP adapter, the 10 ms control tick and the RGB LED
//! indicator, all driving the host-tested `Device` core. No motor GPIO is configured.

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_futures::{join::join3, select::select};
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    Config as NetConfig, IpAddress, IpEndpoint, Ipv4Address, Ipv4Cidr, StackResources,
    StaticConfigV4,
};
use embassy_time::{Instant, Timer};
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, ram, rmt::Rmt, rng::Rng, time::Rate, timer::timg::TimerGroup};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use esp_radio::wifi::{self, ap::AccessPointConfig, AuthenticationMethodConfig, WifiController};
use rovi_m8_setpoint_ros::arbiter::{Config, Effects, State};
use rovi_m8_setpoint_ros::device::Device;
use rovi_m8_setpoint_ros::kinematics::Geometry;
use smart_leds::{SmartLedsWrite, RGB8};

esp_bootloader_esp_idf::esp_app_desc!();

const UDP_PORT: u16 = 7777;
/// Placeholder chassis numbers until M5 measures them.
const GEOMETRY: Geometry = Geometry {
    wheels_radius: 0.05,
    center_projection_sum: 0.2,
    offset_x: 0.0,
    offset_y: 0.0,
    offset_theta: 0.0,
    max_wheel_speed: 40.0,
};

fn now() -> u64 {
    Instant::now().as_millis()
}

fn peer_id(e: IpEndpoint) -> u128 {
    match e.addr {
        IpAddress::Ipv4(a) => (u32::from_be_bytes(a.octets()) as u128) << 16 | e.port as u128,
    }
}

/// One line per event, in the same words as the sim car's log.
fn log(fx: Effects) {
    if let Some(s) = fx.claimed {
        println!("claimed {:?} at_ms={}", s, now());
    }
    if let Some(r) = fx.stop {
        println!("stop {:?} at_ms={}", r, now());
    }
    if fx.released {
        println!("released at_ms={}", now());
    }
}

/// The indicator shows the state, and while driving the direction and speed.
fn color(state: State, wheels: [f32; 4]) -> RGB8 {
    let avg = wheels.iter().sum::<f32>() / 4.0;
    let level = |v: f32| (6.0 + 40.0 * v.abs().min(1.0)) as u8;
    match state {
        State::Open => RGB8 { r: 0, g: 0, b: 0 },
        State::Claimed => RGB8 { r: 0, g: 0, b: 12 },
        State::Armed if avg > 0.01 => RGB8 { r: 0, g: level(avg), b: 0 },
        State::Armed if avg < -0.01 => RGB8 { r: level(avg), g: 0, b: 0 },
        State::Armed => RGB8 { r: 0, g: 6, b: 6 },
        State::Stopped(_) => RGB8 { r: 12, g: 0, b: 0 },
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    let timers = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timers.timer0, p.FROM_CPU_INTR0);
    let seed = Rng::new().random() as u64;
    let password = env!(
        "ROVI_WIFI_PASSWORD",
        "set a private ROVI_WIFI_PASSWORD at build time"
    );
    assert!(
        (8..=63).contains(&password.len()) && password.is_ascii(),
        "WPA2 passphrase: 8..63 ASCII bytes"
    );

    let freq = Rate::from_mhz(80);
    let rmt = Rmt::new(p.RMT, freq).unwrap();
    let mut led =
        RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb>::new_with_memsize(
            esp_hal_smartled::WS2812_TIMING,
            rmt.channel0,
            p.GPIO48,
            2,
            freq,
        )
        .unwrap();
    led.write([RGB8 { r: 0, g: 0, b: 0 }]).unwrap();

    let mut wifi = WifiController::new(p.WIFI, Default::default()).unwrap();
    wifi.set_config(&wifi::Config::AccessPoint(
        AccessPointConfig::default()
            .with_ssid("Rovi M8".try_into().unwrap())
            .with_channel(1)
            .with_max_connections(1)
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                password.try_into().unwrap(),
            )),
    ))
    .unwrap();
    let mut resources = StackResources::<2>::new();
    let (net, mut net_runner) = embassy_net::new(
        wifi::Interface::access_point(),
        NetConfig::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 4, 1), 24),
            gateway: None,
            dns_servers: Default::default(),
        }),
        &mut resources,
        seed,
    );

    let dev = RefCell::new(Device::new(Config::DEFAULT, GEOMETRY));
    println!(
        "M8 board-only AP=Rovi M8 udp=192.168.4.1:{} WPA2 channel=1 lease_ms={}",
        UDP_PORT,
        Config::DEFAULT.lease_ms
    );

    let udp = async {
        let (mut rx_meta, mut tx_meta) = ([PacketMetadata::EMPTY; 4], [PacketMetadata::EMPTY; 4]);
        let (mut rx_buf, mut tx_buf) = ([0u8; 512], [0u8; 512]);
        let mut sock = UdpSocket::new(net, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
        sock.bind(UDP_PORT).unwrap();
        let (mut buf, mut out) = ([0u8; 512], [0u8; 128]);
        loop {
            let Ok((n, meta)) = sock.recv_from(&mut buf).await else {
                continue;
            };
            let r = dev
                .borrow_mut()
                .on_udp(now(), peer_id(meta.endpoint), &buf[..n], &mut out);
            if r.len > 0 {
                let _ = sock.send_to(&out[..r.len], meta).await;
            }
            log(r.effects);
        }
    };
    let control = async {
        let mut last = (State::Open, [0.0f32; 4]);
        loop {
            Timer::after_millis(10).await;
            let at = now();
            let (fx, state, wheels) = {
                let mut d = dev.borrow_mut();
                let fx = d.tick(at);
                (fx, d.arbiter().state(), d.wheels(at))
            };
            log(fx);
            if (state, wheels) != last {
                println!("state {:?} wheels {:?} at_ms={}", state, wheels, at);
                // Only this loop touches the indicator.
                led.write([color(state, wheels)]).unwrap();
                last = (state, wheels);
            }
        }
    };
    let _ = select(join3(net_runner.run(), udp, control), core::future::pending::<()>()).await;
    panic!("runner exited");
}
