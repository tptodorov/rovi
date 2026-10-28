//! Board-only firmware shell: the network (home station or car AP, ADR-0016), the UDP adapter,
//! BLE, the 10 ms control tick, the motor outputs (two TB6612FNG drivers, four channels) and the
//! RGB LED indicator, all driving the host-tested `Device` core.

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_futures::{
    join::{join, join3, join5},
    select::{select, select3, Either, Either3},
};
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    Config as NetConfig, IpAddress, IpEndpoint, Ipv4Address, Ipv4Cidr, StackResources,
    StaticConfigV4,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{with_timeout, Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{AnyPin, Level, Output, OutputConfig},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    ram,
    rmt::Rmt,
    rng::Rng,
    rtc_cntl::{Rtc, RwdtStage, RwdtStageAction, SocResetReason},
    system::{reset_reason, software_reset},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use esp_radio::wifi::{
    self, ap::AccessPointConfig, scan::ScanConfig, sta::StationConfig, AuthenticationMethodConfig,
    WifiController,
};
use rovi_core::arbiter::{Config, Effects, Source, State};
use rovi_core::device::{self, Device};
use rovi_core::events::{self, Event, Fault};
use rovi_core::kinematics::Geometry;
use rovi_core::network::{self as netmode, Mode, Reason, Settings, SettingsError, Step};
use rovi_drive::Drive;
use rovi_motor_control::{Config as MotorConfig, Driver, MotorControl, Pins, CAR_CHANNELS};
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
/// Scans per home attempt before the scan step fails (the core's 1.5 s budget still applies).
const SCAN_TRIES: usize = 3;
/// Association attempts per home attempt before the join step fails (the core's budget still applies).
const JOIN_TRIES: usize = 3;

const HOME_SSID: Option<&str> = option_env!("ROVI_HOME_SSID");
const HOME_PASSWORD: Option<&str> = option_env!("ROVI_HOME_PASSWORD");
const HOME_ROUTER: Option<&str> = option_env!("ROVI_HOME_ROUTER");
/// Home mode needs all three settings and ROS 2 (ADR-0016); a bad set fails the build.
const HOME: bool = match netmode::settings(HOME_SSID, HOME_PASSWORD, HOME_ROUTER) {
    Ok(Settings::Home) => cfg!(feature = "ros"),
    Ok(Settings::None) => false,
    Err(SettingsError::Incomplete) => {
        panic!("set all of ROVI_HOME_SSID, ROVI_HOME_PASSWORD and ROVI_HOME_ROUTER, or none")
    }
    Err(SettingsError::Ssid) => panic!("ROVI_HOME_SSID must be 1..=32 bytes"),
    Err(SettingsError::Passphrase) => panic!("ROVI_HOME_PASSWORD must be 8..=63 printable ASCII"),
    Err(SettingsError::Router) => panic!("ROVI_HOME_ROUTER must be tcp/<IPv4 or host name>:<port>"),
};

/// Survives the software reset that switches station <-> AP (design D2): `SWITCH | reason` boots
/// into fallback with that reason; anything else (power-on, other resets) attempts home.
#[ram(unstable(rtc_fast, persistent))]
static mut NEXT_BOOT: u32 = 0;
const SWITCH: u32 = 0x5256_0000;

/// The fallback reason a mode-switch reset left for this boot, once.
fn take_switch() -> Option<Reason> {
    // SAFETY: read and cleared once at boot, before any task runs.
    let v = unsafe { core::ptr::addr_of_mut!(NEXT_BOOT).replace(0) };
    (v & 0xffff_ff00 == SWITCH)
        .then(|| Reason::from_code(v as u8))
        .flatten()
}

/// Switches station <-> AP by resetting into the other mode: into fallback with `reason`, or
/// into a home attempt. Lets the log ring drain first.
async fn switch_mode(fallback: Option<Reason>) -> ! {
    // SAFETY: written once, just before the reset; nothing else touches it.
    unsafe {
        core::ptr::addr_of_mut!(NEXT_BOOT).write(fallback.map_or(0, |r| SWITCH | r.code() as u32))
    };
    println!(
        "network mode switch: resetting into {}",
        if fallback.is_some() {
            "fallback"
        } else {
            "a home attempt"
        }
    );
    Timer::after_millis(200).await;
    software_reset()
}

/// The home station; `found` pins the strongest access point the scan found (BSSID, channel).
/// On the board this did not shorten the join (0.66-2.8 s either way, 6.1 on 2026-10-26); it
/// keeps the join on the access point the scan saw when a home network has several.
fn station_config(found: Option<([u8; 6], u8)>) -> wifi::Config {
    let config = StationConfig::default()
        .with_ssid(HOME_SSID.unwrap_or("").try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
            HOME_PASSWORD.unwrap_or("").try_into().unwrap(),
        ));
    wifi::Config::Station(match found {
        Some((bssid, channel)) => config.with_bssid(bssid).with_channel(channel),
        None => config,
    })
}

#[cfg(feature = "ros")]
fn ros_up() -> bool {
    ros::LINK_UP.load(core::sync::atomic::Ordering::Relaxed)
}
#[cfg(not(feature = "ros"))]
fn ros_up() -> bool {
    false
}

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
    /// 1 = ARM, 2 = STOP, 3 = Home, 4 = Car network.
    #[characteristic(uuid = "d47a0012-45b2-4d19-8db0-6bd87e9c0001", write, value = 0)]
    control: u8,
    #[characteristic(uuid = "d47a0013-45b2-4d19-8db0-6bd87e9c0001", read, value = [0; 27])]
    capabilities: [u8; 27],
    /// State, stop reason, last error.
    #[characteristic(uuid = "d47a0014-45b2-4d19-8db0-6bd87e9c0001", read, notify, value = [0; 3])]
    status: [u8; 3],
    /// Latency echo: up to 32 bytes written come back unchanged as a notification.
    #[characteristic(uuid = "d47a0015-45b2-4d19-8db0-6bd87e9c0001", write_without_response, notify, value = [0; 32])]
    echo: [u8; 32],
    /// Network information: mode, pending command, fallback reason, 0, station IPv4.
    #[characteristic(uuid = "d47a0016-45b2-4d19-8db0-6bd87e9c0001", read, notify, value = [0; 8])]
    network: [u8; 8],
}

async fn until(mut cond: impl FnMut() -> bool) {
    while !cond() {
        Timer::after_millis(20).await;
    }
}

/// MCPWM period in counts, so the compare value is the duty in permille (20 kHz, as M1).
const PWM_PERIOD: u16 = 999;
/// The RTC watchdog resets the chip if the control tick stops feeding it (ADR-0014).
const RWDT_MS: u64 = 1000;

fn reset_name() -> &'static str {
    use SocResetReason as R;
    match reset_reason() {
        Some(R::ChipPowerOn) => "ChipPowerOn",
        Some(R::SysBrownOut) => "SysBrownOut",
        Some(R::SysRtcWdt) => "SysRtcWdt",
        Some(R::CoreRtcWdt) => "CoreRtcWdt",
        Some(R::CpuRtcWdt) => "CpuRtcWdt",
        Some(R::CoreMwdt0) | Some(R::CoreMwdt1) | Some(R::CpuMwdt0) | Some(R::CpuMwdt1) => "Mwdt",
        Some(R::SysSuperWdt) => "SysSuperWdt",
        Some(R::CoreSw) | Some(R::CpuSw) => "Sw",
        Some(R::CoreUsbUart) | Some(R::CoreUsbJtag) => "Usb",
        Some(R::SysClkGlitch) | Some(R::CorePwrGlitch) => "Glitch",
        Some(_) => "Other",
        None => "Unknown",
    }
}

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

/// The motor pins for motor control: each driver's STBY, each bridge's IN1/IN2, and the four PWM
/// compare values behind one closure (the MCPWM pins have distinct types). The period is
/// [`PWM_PERIOD`] counts, so a compare value of 1000 never matches and holds PWM high (coast).
struct MotorPins<P> {
    stby: [Output<'static>; 2],
    dirs: [(Output<'static>, Output<'static>); 4],
    pwm: P,
}

impl<P: FnMut(usize, u16)> Driver for MotorPins<P> {
    type Error = core::convert::Infallible;

    fn set_stby(&mut self, driver: usize, high: bool) -> Result<(), Self::Error> {
        if let Some(pin) = self.stby.get_mut(driver) {
            pin.set_level(Level::from(high));
        }
        Ok(())
    }

    fn set_bridge(&mut self, bridge: usize, pins: Pins) -> Result<(), Self::Error> {
        (self.pwm)(bridge, pins.pwm_pm);
        if let Some((in1, in2)) = self.dirs.get_mut(bridge) {
            in1.set_level(Level::from(pins.in1));
            in2.set_level(Level::from(pins.in2));
        }
        Ok(())
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
    // Motors first: STBY low before anything else runs (the TB6612's pull-down holds it low until
    // here). Pins as M4 WIRING: each driver's 7 control pins are 7 consecutive header pins, so one
    // ribbon cable per driver (driver 1: left header L4-L10, GPIO4-7 and 15-17; driver 2: left header
    // L14-L20, GPIO46 and 9-14). Each driver has its own STBY; both switch together.
    let low = |pin: AnyPin<'static>| Output::new(pin, Level::Low, OutputConfig::default());
    let stby = [low(p.GPIO7.into()), low(p.GPIO11.into())]; // driver 1, driver 2
    let dirs = [
        (low(p.GPIO6.into()), low(p.GPIO5.into())), // driver 1 A: AIN1, AIN2
        (low(p.GPIO15.into()), low(p.GPIO16.into())), // driver 1 B: BIN1, BIN2
        (low(p.GPIO10.into()), low(p.GPIO9.into())), // driver 2 A: AIN1, AIN2
        (low(p.GPIO12.into()), low(p.GPIO13.into())), // driver 2 B: BIN1, BIN2
    ];
    let pwm_clock = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();
    let mut mcpwm = McPwm::new(p.MCPWM0, pwm_clock);
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    mcpwm.operator1.set_timer(&mcpwm.timer0);
    let (mut pwm_1a, mut pwm_1b) = mcpwm.operator0.with_pins(
        p.GPIO4,
        PwmPinConfig::UP_ACTIVE_HIGH,
        p.GPIO17,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    let (mut pwm_2a, mut pwm_2b) = mcpwm.operator1.with_pins(
        p.GPIO46,
        PwmPinConfig::UP_ACTIVE_HIGH,
        p.GPIO14,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    pwm_1a.set_timestamp(0);
    pwm_1b.set_timestamp(0);
    pwm_2a.set_timestamp(0);
    pwm_2b.set_timestamp(0);
    mcpwm.timer0.start(
        pwm_clock
            .timer_clock_with_frequency(PWM_PERIOD, PwmWorkingMode::Increase, Rate::from_khz(20))
            .unwrap(),
    );
    // Only the coordinator touches the motor pins, through motor control: the product profile
    // (40-70 % duty, M6 O1) and the car's bridge mapping (O8).
    let pwm = move |bridge: usize, pm: u16| match bridge {
        0 => pwm_1a.set_timestamp(pm),
        1 => pwm_1b.set_timestamp(pm),
        2 => pwm_2a.set_timestamp(pm),
        3 => pwm_2b.set_timestamp(pm),
        _ => {}
    };
    let pins = MotorPins { stby, dirs, pwm };
    let Ok(motor) = MotorControl::new(pins, MotorConfig::PRODUCT, CAR_CHANNELS) else {
        unreachable!("motor pin writes cannot fail");
    };
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

    // One network per boot (design D2): the home station when built with home settings and not
    // just switched to fallback, else the car AP as before. Switching resets into the other.
    let switched = take_switch();
    let home_boot = HOME && switched.is_none();
    let mut wifi = WifiController::new(p.WIFI, Default::default()).unwrap();
    static RESOURCES: static_cell::StaticCell<StackResources<6>> = static_cell::StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());
    let (net, mut net_runner) = if home_boot {
        wifi.set_config(&station_config(None)).unwrap();
        embassy_net::new(
            wifi::Interface::station(),
            NetConfig::dhcpv4(Default::default()),
            resources,
            seed,
        )
    } else {
        wifi.set_config(&ap_config(password)).unwrap();
        embassy_net::new(
            wifi::Interface::access_point(),
            NetConfig::ipv4_static(StaticConfigV4 {
                address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 4, 1), 24),
                gateway: None,
                dns_servers: Default::default(),
            }),
            resources,
            seed,
        )
    };

    let connector =
        esp_radio::ble::controller::BleConnector::new(p.BT, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);
    // Two connections: one owner and one read-only watcher (ADR-0016).
    let mut ble_resources: HostResources<DefaultPacketPool, 2, 1> = HostResources::new();
    let stack = trouble_host::new(controller, &mut ble_resources)
        .set_random_address(Address::random([0xff, 0x9f, 0x1a, 0x05, 0xe4, 0xfe]))
        .build();
    let mut ble_runner = stack.runner();
    let peripheral = stack.peripheral();
    let server = RoviServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Rovi M8",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();

    // The profile is checked on the host (`Geometry::CAR` with `Config::DEFAULT`); an invalid one
    // halts here with both drivers still in standby, before anything can claim the car.
    // A random session seed per boot keeps a client's packets from before a reboot out of the
    // new sessions (not authentication).
    let Ok(mut device) = Device::new(Config::DEFAULT, Geometry::CAR, Rng::new().random()) else {
        panic!("invalid device profile");
    };
    match switched {
        Some(reason) => device.start_network_fallback(HOME, reason),
        None => device.start_network(now(), HOME),
    }
    if matches!(motor.fault(), Some(rovi_motor_control::Fault::Config(_))) {
        logger::emit(Event::Fault(Fault::MotorConfig));
    }
    let mut drive = Drive::new(motor, &mut device);
    let dev = RefCell::new(device);
    #[cfg(feature = "motor-bench")]
    println!("{}", rovi_drive::bench::BANNER);
    if home_boot {
        println!(
            "home mode attempt: station SSID={} router={} udp={} lease_ms={}",
            HOME_SSID.unwrap_or(""),
            HOME_ROUTER.unwrap_or(""),
            UDP_PORT,
            Config::DEFAULT.lease_ms
        );
    } else {
        println!(
            "M8 board-only AP=Rovi M8 udp=192.168.4.1:{} WPA2 channel=1 lease_ms={}",
            UDP_PORT,
            Config::DEFAULT.lease_ms
        );
    }
    logger::emit(Event::Boot {
        reset: Some(reset_name()),
    });

    let udp = async {
        let (mut rx_meta, mut tx_meta) = ([PacketMetadata::EMPTY; 4], [PacketMetadata::EMPTY; 4]);
        let (mut rx_buf, mut tx_buf) = ([0u8; 512], [0u8; 512]);
        let mut sock = UdpSocket::new(net, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
        sock.bind(UDP_PORT).unwrap();
        let (mut buf, mut out) = ([0u8; 512], [0u8; 128]);
        let mut owner_endpoint = None;
        // Recent peers, so a taken-over UDP owner can be told (spec: Takeover notice).
        let mut peers: [Option<(u128, IpEndpoint)>; 4] = [None; 4];
        loop {
            if let Either::First(Ok((n, meta))) =
                select(sock.recv_from(&mut buf), Timer::after_millis(10)).await
            {
                let id = peer_id(meta.endpoint);
                if !peers.iter().flatten().any(|(p, _)| *p == id) {
                    peers.rotate_right(1);
                    peers[0] = Some((id, meta.endpoint));
                }
                let r = dev.borrow_mut().on_udp(now(), id, &buf[..n], &mut out);
                if let Some((source, cmd, error)) = dev.borrow_mut().take_mode_log() {
                    logger::emit(Event::ModeCmd { source, cmd, error });
                }
                if dev
                    .borrow()
                    .arbiter()
                    .owner()
                    .is_some_and(|o| o.source == Source::Udp && o.id == peer_id(meta.endpoint))
                {
                    owner_endpoint = Some(meta.endpoint);
                }
                if r.len > 0 {
                    let _ = sock.send_to(&out[..r.len], meta).await;
                }
                log(r.effects);
            }
            let notice = dev.borrow_mut().take_takeover_notice(Source::Udp);
            if let Some(n) = notice {
                let len = n.udp_frame(&mut out);
                if let Some((_, to)) = peers.iter().flatten().find(|(p, _)| *p == n.id) {
                    let _ = sock.send_to(&out[..len], *to).await;
                }
            }
            let pending = dev.borrow_mut().udp_status_notification(&mut out);
            if let Some((peer, n)) = pending {
                if let Some(to) = owner_endpoint.filter(|a| peer_id(*a) == peer) {
                    let _ = sock.send_to(&out[..n], to).await;
                }
            }
        }
    };
    // BLE: two connection slots (an owner and a read-only watcher, ADR-0016). A central claims
    // with its first Setpoint or Control write, not by connecting. While a Wi-Fi source owns the
    // car in fallback mode, do not advertise; in home mode BLE always advertises, so it can take
    // over. One slot advertises at a time.
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
    let advertising = Mutex::<NoopRawMutex, _>::new(peripheral);
    let slot = async || loop {
        // Quiet during a home attempt too (the core's rule): advertising slows the scan.
        let ble_ok = || dev.borrow().radio_allowed(Source::Ble);
        until(ble_ok).await;
        let accepted = {
            let mut peripheral = advertising.lock().await;
            select(
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
                until(|| !ble_ok()),
            )
            .await
        };
        let Either::First(Some(raw)) = accepted else {
            continue;
        };
        let Ok(conn) = raw.with_attribute_server(&server) else {
            continue;
        };
        let id = conn.raw().handle().raw();
        // The token, not the reusable handle, identifies this connection to the device.
        let ble = dev.borrow_mut().on_ble_attach(now(), id);
        logger::emit(Event::Ble {
            id,
            mtu: Some(conn.raw().att_mtu()),
        });
        let caps = dev.borrow().capabilities();
        let _ = conn.set(&server.rovi.capabilities, &caps);
        // Status and network are readable right after connecting, not only after a notify tick.
        let mut last_status = dev.borrow().ble_status();
        let _ = conn.set(&server.rovi.status, &last_status);
        let mut last_network = dev.borrow().network_info();
        let _ = conn.set(&server.rovi.network, &last_network);
        // It has owned the car: losing ownership disconnects it (a watcher stays).
        let owned = core::cell::Cell::new(false);
        // One schedule for the whole connection: a timer recreated per loop pass would never
        // fire while the central writes setpoints faster than every 100 ms.
        let mut tick = Ticker::every(Duration::from_millis(100));
        let mut mtu_seen = conn.raw().att_mtu();
        loop {
            match select3(
                conn.next(),
                until(|| owned.get() && !dev.borrow().ble_owns(ble)),
                tick.next(),
            )
            .await
            {
                Either3::First(GattConnectionEvent::Disconnected { .. }) => break,
                Either3::First(GattConnectionEvent::Gatt {
                    event: GattEvent::Read(r),
                }) => {
                    let _ = conn.set(&server.rovi.status, &dev.borrow().ble_status());
                    let _ = conn.set(&server.rovi.network, &dev.borrow().network_info());
                    if let Ok(reply) = r.accept() {
                        reply.send().await;
                    }
                }
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
                        let fx = dev.borrow_mut().on_ble_setpoint(now(), ble, &data[..n]);
                        log(fx);
                        let _ = conn.set(&server.rovi.status, &dev.borrow().ble_status());
                        if let Ok(reply) = w.accept() {
                            reply.send().await;
                        }
                    } else if handle == server.rovi.echo.handle {
                        let mut data = [0u8; device::ECHO_MAX];
                        let reply_len = w.with_data(|off, b| device::ble_echo(off, b, &mut data));
                        if let Ok(reply) = w.accept() {
                            reply.send().await;
                        }
                        if let Some(n) = reply_len {
                            let _ = server.rovi.echo.notify_raw(&conn, &data[..n], false).await;
                        }
                    } else if handle == server.rovi.control.handle {
                        let cmd =
                            w.with_data(|off, b| if off == 0 && b.len() == 1 { b[0] } else { 0 });
                        let res = dev.borrow_mut().on_ble_control(now(), ble, cmd);
                        if let Some((source, cmd, error)) = dev.borrow_mut().take_mode_log() {
                            logger::emit(Event::ModeCmd { source, cmd, error });
                        }
                        let _ = conn.set(&server.rovi.status, &dev.borrow().ble_status());
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
                Either3::First(GattConnectionEvent::Gatt { event }) => {
                    // Other ATT reads also use the current value before their default reply.
                    let _ = conn.set(&server.rovi.status, &dev.borrow().ble_status());
                    drop(event);
                }
                Either3::First(_) => {}
                Either3::Second(()) => {
                    conn.raw().disconnect();
                    break;
                }
                Either3::Third(()) => {
                    // The central's MTU exchange changes it from the value at connect.
                    let mtu = conn.raw().att_mtu();
                    if mtu != mtu_seen {
                        mtu_seen = mtu;
                        logger::emit(Event::Mtu { id, mtu });
                    }
                }
            }
            if dev.borrow().ble_owns(ble) {
                owned.set(true);
            }
            let status = dev.borrow().ble_status();
            let _ = conn.set(&server.rovi.status, &status);
            if status != last_status {
                last_status = status;
                let _ = server.rovi.status.notify(&conn, &status, true).await;
            }
            let network = dev.borrow().network_info();
            if network != last_network {
                last_network = network;
                let _ = server.rovi.network.notify(&conn, &network, true).await;
            }
        }
        let fx = dev.borrow_mut().on_ble_disconnect(now(), ble);
        log(fx);
        logger::emit(Event::Ble { id, mtu: None });
    };
    let ble = join(slot(), slot());
    let mut rtc = Rtc::new(p.RTC_TIMER);
    rtc.rwdt.set_timeout(
        RwdtStage::Stage0,
        esp_hal::time::Duration::from_millis(RWDT_MS),
    );
    rtc.rwdt
        .set_stage_action(RwdtStage::Stage0, RwdtStageAction::ResetSystem);
    rtc.rwdt.enable();
    let control = async {
        let mut last = (State::Open, [0.0f32; 4]);
        loop {
            Timer::after_millis(10).await;
            let at = now();
            // The device's timers, then the motor pins (rovi-drive): a stop reaches STBY in the
            // tick that processes it, and a motor fault raises an out-of-limits stop.
            let (tick, state, wheels) = {
                let mut d = dev.borrow_mut();
                #[cfg(not(feature = "motor-bench"))]
                let tick = drive.tick(&mut d, at);
                #[cfg(feature = "motor-bench")]
                let tick = drive.tick_motor_bench(&mut d, at);
                #[cfg(not(feature = "motor-bench"))]
                let wheels = d.wheels(at);
                #[cfg(feature = "motor-bench")]
                let wheels = rovi_drive::bench::wheels(&d, at);
                (tick, d.arbiter().state(), wheels)
            };
            log(tick.effects);
            if let Some(f) = tick.fault {
                logger::emit(Event::Fault(f));
            }
            rtc.rwdt.feed();
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
    // The network supervisor (ADR-0016). Fallback boot: the car AP is off while BLE owns the car
    // and back when it is released, as before; a Home command resets into a home attempt. Home
    // boot: runs the scan, join, DHCP and router steps under the core's budgets, turns the
    // station off while BLE owns the car, reports a lost link, and resets into fallback.
    let supervisor = async {
        let mut last = None;
        let mut ap_up = true;
        let mut station_on = false;
        loop {
            let (mode, reason, allowed) = {
                let d = dev.borrow();
                (d.network().mode(), d.network().reason(), d.wifi_allowed())
            };
            if last != Some((mode, reason)) {
                logger::emit(Event::Net { mode, reason });
                last = Some((mode, reason));
            }
            match mode {
                Mode::Fallback if home_boot => switch_mode(Some(reason)).await,
                Mode::Fallback => {
                    if allowed != ap_up {
                        ap_up = allowed;
                        let config = if ap_up {
                            ap_config(password)
                        } else {
                            ap_off_config()
                        };
                        match wifi.set_config(&config) {
                            Ok(()) => logger::emit(Event::Wifi { ap_on: ap_up }),
                            Err(e) => {
                                println!("wifi AP switch failed: {:?}", e);
                                logger::emit(Event::Fault(Fault::WifiAp));
                            }
                        }
                    }
                    Timer::after_millis(20).await;
                }
                _ if !home_boot => switch_mode(None).await,
                Mode::Attempting(step) => {
                    let started = now();
                    let budget = Duration::from_millis(match step {
                        Step::Scan => netmode::SCAN_BUDGET_MS,
                        _ => netmode::JOIN_BUDGET_MS,
                    });
                    let run = async {
                        match step {
                            Step::Scan => {
                                let cfg = ScanConfig::default()
                                    .with_ssid(HOME_SSID.unwrap_or("").try_into().unwrap());
                                // One active scan takes ~160 ms and missed the access point about 1 in 30
                                // times on the board (2026-10-28); the budget allows a few tries.
                                let mut best = None;
                                for _ in 0..SCAN_TRIES {
                                    best = match with_timeout(budget, wifi.scan_async(&cfg)).await {
                                        Ok(Ok(found)) => found
                                            .iter()
                                            .max_by_key(|ap| ap.signal_strength)
                                            .map(|ap| (ap.bssid, ap.channel)),
                                        _ => None,
                                    };
                                    if best.is_some() {
                                        break;
                                    }
                                }
                                if let Some(ap) = best {
                                    let _ = wifi.set_config(&station_config(Some(ap)));
                                }
                                best.is_some()
                            }
                            Step::Join => {
                                // An association sometimes fails at once (1 of ~25 boots on the board,
                                // 2026-10-29); try again inside the budget. A wrong passphrase makes the
                                // driver block for the whole budget instead, so it still fails.
                                for _ in 0..JOIN_TRIES {
                                    station_on = matches!(
                                        with_timeout(budget, wifi.connect_async()).await,
                                        Ok(Ok(_))
                                    );
                                    if station_on {
                                        break;
                                    }
                                }
                                station_on
                            }
                            Step::Dhcp => with_timeout(budget, net.wait_config_up()).await.is_ok(),
                            Step::Router => with_timeout(budget, until(ros_up)).await.is_ok(),
                        }
                    };
                    // The core ends the attempt when its budget runs out; stop the step then too.
                    let left = until(|| dev.borrow().network().mode() != Mode::Attempting(step));
                    let ok = matches!(select(run, left).await, Either::First(true));
                    let ip = net.config_v4().map(|c| c.address.address().octets());
                    let mut d = dev.borrow_mut();
                    d.on_network_step(now(), step, ok);
                    if step == Step::Dhcp {
                        d.set_station_ip(ip.filter(|_| ok));
                    }
                    logger::emit(Event::NetStep {
                        step,
                        ok,
                        took_ms: now().saturating_sub(started) as u32,
                    });
                }
                Mode::Home if !allowed => {
                    // BLE owns the car: the station is off until it ends (the core reattempts).
                    if station_on {
                        let _ = wifi.disconnect_async().await;
                        station_on = false;
                        dev.borrow_mut().set_station_ip(None);
                    }
                    Timer::after_millis(20).await;
                }
                Mode::Home => {
                    let changed = until(|| {
                        let d = dev.borrow();
                        d.network().mode() != Mode::Home || !d.wifi_allowed()
                    });
                    if let Either::First(_) =
                        select(wifi.wait_for_disconnect_async(), changed).await
                    {
                        station_on = false;
                        let mut d = dev.borrow_mut();
                        d.set_station_ip(None);
                        d.on_station_lost(now());
                    }
                }
            }
        }
    };
    #[cfg(feature = "ros")]
    let ros = ros::run(
        &dev,
        net,
        if home_boot {
            HOME_ROUTER.unwrap_or(ros::ROUTER)
        } else {
            ros::ROUTER
        },
    );
    #[cfg(not(feature = "ros"))]
    let ros = core::future::pending::<()>();
    let _ = select(
        join5(
            net_runner.run(),
            udp,
            control,
            select(ble_runner.run(), ble),
            join3(ros, supervisor, logger::run()),
        ),
        core::future::pending::<()>(),
    )
    .await;
    panic!("runner exited");
}
