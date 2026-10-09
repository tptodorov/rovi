//! Board-only firmware shell: the car AP, the UDP adapter, the 10 ms control tick and the RGB LED
//! indicator, all driving the host-tested `Device` core. No motor GPIO is configured.

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_futures::{
    join::{join3, join5},
    select::{select, select3, Either, Either3},
};
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
use rovi_m8_setpoint_ros::arbiter::{Config, Effects, Error, Source, State};
use rovi_m8_setpoint_ros::device::Device;
use rovi_m8_setpoint_ros::events::{self, Event, Fault};
use rovi_m8_setpoint_ros::kinematics::Geometry;
use smart_leds::{SmartLedsWrite, RGB8};
use trouble_host::prelude::*;

esp_bootloader_esp_idf::esp_app_desc!();

#[path = "logger.rs"]
mod logger;
#[cfg(feature = "ros")]
#[path = "ros.rs"]
mod ros;

#[cfg(feature = "ros")]
getrandom::register_custom_getrandom!(random_bytes);
#[cfg(feature = "ros")]
fn random_bytes(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    Rng::new().read(bytes);
    Ok(())
}

const UDP_PORT: u16 = 7777;

/// New UUIDs; not M2-compatible. The Setpoint value is `seq u32` + the 68-byte CDR (72 bytes),
/// which needs an ATT MTU of at least 75.
#[gatt_server]
struct RoviServer {
    rovi: RoviService,
}
#[gatt_service(uuid = "d47a0010-45b2-4d19-8db0-6bd87e9c0001")]
struct RoviService {
    #[characteristic(uuid = "d47a0011-45b2-4d19-8db0-6bd87e9c0001", write_without_response, value = [0; 72])]
    setpoint: [u8; 72],
    /// 1 = ARM, 2 = STOP.
    #[characteristic(uuid = "d47a0012-45b2-4d19-8db0-6bd87e9c0001", write, value = 0)]
    control: u8,
    #[characteristic(uuid = "d47a0013-45b2-4d19-8db0-6bd87e9c0001", read, value = [0; 27])]
    capabilities: [u8; 27],
    /// State, stop reason, last error.
    #[characteristic(uuid = "d47a0014-45b2-4d19-8db0-6bd87e9c0001", read, notify, value = [0; 3])]
    status: [u8; 3],
}

async fn until(mut cond: impl FnMut() -> bool) {
    while !cond() {
        Timer::after_millis(20).await;
    }
}
/// Placeholder chassis numbers until M5 measures them.
const GEOMETRY: Geometry = Geometry {
    wheels_radius: 0.05,
    center_projection_sum: 0.2,
    offset_x: 0.0,
    offset_y: 0.0,
    offset_theta: 0.0,
    max_wheel_speed: 40.0,
};

fn ap_config(password: &'static str) -> wifi::Config {
    wifi::Config::AccessPoint(
        AccessPointConfig::default()
            .with_ssid("Rovi M8".try_into().unwrap())
            .with_channel(1)
            .with_max_connections(1)
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                password.try_into().unwrap(),
            )),
    )
}

/// esp-radio has no AP stop, but a mode change stops the driver: an idle station config (never
/// connected) is how the AP goes off while BLE owns the car.
fn ap_off_config() -> wifi::Config {
    wifi::Config::Station(
        wifi::sta::StationConfig::default().with_ssid("rovi-off".try_into().unwrap()),
    )
}

fn now() -> u64 {
    Instant::now().as_millis()
}

fn peer_id(e: IpEndpoint) -> u128 {
    match e.addr {
        IpAddress::Ipv4(a) => (u32::from_be_bytes(a.octets()) as u128) << 16 | e.port as u128,
    }
}

/// Queues the events of `fx` for the log (`logger.rs`, same lines as the sim car's).
fn log(fx: Effects) {
    events::effects(fx, logger::emit);
}

/// The indicator shows the state, and while driving the direction and speed.
fn color(state: State, wheels: [f32; 4]) -> RGB8 {
    let avg = wheels.iter().sum::<f32>() / 4.0;
    let level = |v: f32| (6.0 + 40.0 * v.abs().min(1.0)) as u8;
    match state {
        State::Open => RGB8 { r: 0, g: 0, b: 0 },
        State::Claimed => RGB8 { r: 0, g: 0, b: 12 },
        State::Armed if avg > 0.01 => RGB8 {
            r: 0,
            g: level(avg),
            b: 0,
        },
        State::Armed if avg < -0.01 => RGB8 {
            r: level(avg),
            g: 0,
            b: 0,
        },
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
    wifi.set_config(&ap_config(password)).unwrap();
    static RESOURCES: static_cell::StaticCell<StackResources<6>> = static_cell::StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());
    let (net, mut net_runner) = embassy_net::new(
        wifi::Interface::access_point(),
        NetConfig::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 4, 1), 24),
            gateway: None,
            dns_servers: Default::default(),
        }),
        resources,
        seed,
    );

    let connector =
        esp_radio::ble::controller::BleConnector::new(p.BT, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);
    let mut ble_resources: HostResources<DefaultPacketPool, 1, 1> = HostResources::new();
    let stack = trouble_host::new(controller, &mut ble_resources)
        .set_random_address(Address::random([0xff, 0x9f, 0x1a, 0x05, 0xe4, 0xfe]))
        .build();
    let mut ble_runner = stack.runner();
    let mut peripheral = stack.peripheral();
    let server = RoviServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Rovi M8",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();

    let dev = RefCell::new(Device::new(Config::DEFAULT, GEOMETRY));
    println!(
        "M8 board-only AP=Rovi M8 udp=192.168.4.1:{} WPA2 channel=1 lease_ms={}",
        UDP_PORT,
        Config::DEFAULT.lease_ms
    );
    logger::emit(Event::Boot);

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
    // BLE: connecting is the claim. While another source owns the car, do not advertise (the
    // Wi-Fi side is refused by the arbiter, since esp-radio has no AP stop in this beta).
    let ble = async {
        let mut adv_data = [0; 31];
        let adv_len = AdStructure::encode_slice(
            &[AdStructure::Flags(
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
            )],
            &mut adv_data,
        )
        .unwrap();
        let mut scan_data = [0; 31];
        let scan_len = AdStructure::encode_slice(
            &[AdStructure::CompleteLocalName(b"Rovi M8")],
            &mut scan_data,
        )
        .unwrap();
        loop {
            until(|| dev.borrow().radio_allowed(Source::Ble)).await;
            let accepted = select(
                async {
                    let advertiser = peripheral
                        .advertise(
                            &Default::default(),
                            Advertisement::ConnectableScannableUndirected {
                                adv_data: &adv_data[..adv_len],
                                scan_data: &scan_data[..scan_len],
                            },
                        )
                        .await
                        .ok()?;
                    advertiser.accept().await.ok()
                },
                until(|| !dev.borrow().radio_allowed(Source::Ble)),
            )
            .await;
            let Either::First(Some(raw)) = accepted else {
                continue;
            };
            let Ok(conn) = raw.with_attribute_server(&server) else {
                continue;
            };
            let id = conn.raw().handle().raw();
            let claim = dev.borrow_mut().on_ble_connect(now(), id);
            match claim {
                Ok(fx) => log(fx),
                Err(_) => {
                    conn.raw().disconnect();
                    continue;
                }
            }
            logger::emit(Event::Ble {
                id,
                mtu: Some(conn.raw().att_mtu()),
            });
            let caps = dev.borrow().capabilities();
            let _ = conn.set(&server.rovi.capabilities, &caps);
            let mut last_status = [255u8; 3];
            loop {
                match select3(
                    conn.next(),
                    until(|| !dev.borrow().ble_owns(id)),
                    Timer::after_millis(100),
                )
                .await
                {
                    Either3::First(GattConnectionEvent::Disconnected { .. }) => break,
                    Either3::First(GattConnectionEvent::Gatt {
                        event: GattEvent::Write(w),
                    }) => {
                        let handle = w.handle();
                        if handle == server.rovi.setpoint.handle {
                            let mut data = [0u8; 80];
                            let n = w.with_data(|off, b| {
                                if off == 0 && b.len() <= data.len() {
                                    data[..b.len()].copy_from_slice(b);
                                    b.len()
                                } else {
                                    0
                                }
                            });
                            let fx = dev.borrow_mut().on_ble_setpoint(now(), id, &data[..n]);
                            log(fx);
                            if let Ok(reply) = w.accept() {
                                reply.send().await;
                            }
                        } else if handle == server.rovi.control.handle {
                            let cmd =
                                w.with_data(|off, b| (off == 0 && b.len() == 1).then(|| b[0]));
                            let res = cmd
                                .ok_or(Error::Invalid)
                                .and_then(|c| dev.borrow_mut().on_ble_control(now(), id, c));
                            match res {
                                Ok(fx) => {
                                    log(fx);
                                    if let Ok(reply) = w.accept() {
                                        reply.send().await;
                                    }
                                }
                                Err(_) => {
                                    if let Ok(reply) = w.reject(AttErrorCode::VALUE_NOT_ALLOWED) {
                                        reply.send().await;
                                    }
                                }
                            }
                        }
                    }
                    Either3::First(_) => {}
                    Either3::Second(()) => {
                        conn.raw().disconnect();
                        break;
                    }
                    Either3::Third(()) => {
                        let status = dev.borrow().status(None);
                        if status != last_status {
                            last_status = status;
                            let _ = server.rovi.status.notify(&conn, &status, true).await;
                        }
                    }
                }
            }
            let fx = dev.borrow_mut().on_ble_disconnect(now(), id);
            log(fx);
            logger::emit(Event::Ble { id, mtu: None });
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
                logger::emit(Event::State {
                    state,
                    wheels_pm: events::permille(wheels),
                });
                // Only this loop touches the indicator.
                led.write([color(state, wheels)]).unwrap();
                last = (state, wheels);
            }
        }
    };
    // Wi-Fi is off while BLE owns the car and back when the car is released.
    let wifi_gate = async {
        let mut up = true;
        loop {
            until(|| dev.borrow().wifi_allowed() != up).await;
            up = !up;
            let config = if up {
                ap_config(password)
            } else {
                ap_off_config()
            };
            match wifi.set_config(&config) {
                Ok(()) => logger::emit(Event::Wifi { ap_on: up }),
                Err(e) => {
                    println!("wifi AP switch failed: {:?}", e);
                    logger::emit(Event::Fault(Fault::WifiAp));
                }
            }
        }
    };
    #[cfg(feature = "ros")]
    let ros = ros::run(&dev, net);
    #[cfg(not(feature = "ros"))]
    let ros = core::future::pending::<()>();
    let _ = select(
        join5(
            net_runner.run(),
            udp,
            control,
            select(ble_runner.run(), ble),
            join3(ros, wifi_gate, logger::run()),
        ),
        core::future::pending::<()>(),
    )
    .await;
    panic!("runner exited");
}
