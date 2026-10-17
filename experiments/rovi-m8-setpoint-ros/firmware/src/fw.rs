//! Board-only firmware shell: the car AP, the UDP adapter, the 10 ms control tick, the motor
//! outputs (two TB6612FNG drivers, four channels) and the RGB LED indicator, all driving the
//! host-tested `Device` core.

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
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{AnyPin, Level, Output, OutputConfig},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    ram,
    rmt::Rmt,
    rng::Rng,
    rtc_cntl::{Rtc, RwdtStage, RwdtStageAction, SocResetReason},
    system::reset_reason,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use esp_radio::wifi::{self, ap::AccessPointConfig, AuthenticationMethodConfig, WifiController};
use rovi_m8_setpoint_ros::arbiter::{Config, Effects, Error, Source, State};
use rovi_m8_setpoint_ros::device::{self, Device};
use rovi_m8_setpoint_ros::events::{self, Event, Fault};
use rovi_m8_setpoint_ros::kinematics::Geometry;
use rovi_m8_setpoint_ros::motor::{self, Channel};
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
    /// Latency echo: up to 32 bytes written come back unchanged as a notification.
    #[characteristic(uuid = "d47a0015-45b2-4d19-8db0-6bd87e9c0001", write_without_response, notify, value = [0; 32])]
    echo: [u8; 32],
}

async fn until(mut cond: impl FnMut() -> bool) {
    while !cond() {
        Timer::after_millis(20).await;
    }
}
/// M5 dimensions measured 2026-10-17; speed normalization still awaits calibration.
const GEOMETRY: Geometry = Geometry {
    wheels_radius: 0.030,
    center_projection_sum: 0.1275, // (0.115 m wheelbase + 0.140 m track) / 2
    offset_x: 0.0,
    offset_y: 0.0,
    offset_theta: 0.0,
    max_wheel_speed: 40.0, // Placeholder, not the unloaded speed measured at 70% duty.
};

/// Duty range for the wheels (M4 step 2, 2026-10-16/17): the mounted TT motors need about 35 %
/// to start with all four connected, so any command outside the 2 % deadband starts at 40 %; a
/// normalized wheel command of 1.0 is 100 %. With M5's measured radius and the still
/// placeholder speed normalization, `vx` 1.0 gives a wheel command of 5/6, so 90 % duty.
const MOTOR: motor::Config = motor::Config {
    min_duty_pm: 400,
    max_duty_pm: 1000,
    deadband: 0.02,
};
/// Bridge order: driver 1 A, driver 1 B, driver 2 A, driver 2 B (M4 WIRING). Driver 1 takes the
/// left wheels, driver 2 the right. Inversion is a placeholder until M4 step 2 records which
/// motors turn backwards.
const CHANNELS: [Channel; 4] = [
    Channel {
        wheel: 0, // FL
        invert: false,
    },
    Channel {
        wheel: 3, // RL
        invert: false,
    },
    Channel {
        wheel: 1, // FR
        invert: false,
    },
    Channel {
        wheel: 2, // RR
        invert: false,
    },
];
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
    let mut stby = [low(p.GPIO7.into()), low(p.GPIO11.into())]; // driver 1, driver 2
    let mut dirs = [
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
    logger::emit(Event::Boot {
        reset: Some(reset_name()),
    });

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
            // Status is readable right after connecting, not only after the first notify tick.
            let mut last_status = dev.borrow().ble_status();
            let _ = conn.set(&server.rovi.status, &last_status);
            // One schedule for the whole connection: a timer recreated per loop pass would never
            // fire while the central writes setpoints faster than every 100 ms.
            let mut tick = Ticker::every(Duration::from_millis(100));
            let mut mtu_seen = conn.raw().att_mtu();
            loop {
                match select3(
                    conn.next(),
                    until(|| !dev.borrow().ble_owns(id)),
                    tick.next(),
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
                        } else if handle == server.rovi.echo.handle {
                            let mut data = [0u8; device::ECHO_MAX];
                            let n = w.with_data(|off, b| {
                                if off == 0 && device::echo(b).is_some() {
                                    data[..b.len()].copy_from_slice(b);
                                    b.len()
                                } else {
                                    0
                                }
                            });
                            if let Ok(reply) = w.accept() {
                                reply.send().await;
                            }
                            if n > 0 {
                                let _ = server.rovi.echo.notify_raw(&conn, &data[..n], false).await;
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
                        // The central's MTU exchange changes it from the value at connect.
                        let mtu = conn.raw().att_mtu();
                        if mtu != mtu_seen {
                            mtu_seen = mtu;
                            logger::emit(Event::Mtu { id, mtu });
                        }
                        let status = dev.borrow().ble_status();
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
            let (fx, state, wheels) = {
                let mut d = dev.borrow_mut();
                let fx = d.tick(at);
                (fx, d.arbiter().state(), d.wheels(at))
            };
            log(fx);
            // Only this loop touches the motor pins: STBY drops before the bridges change and
            // rises after them.
            let out = motor::output(&MOTOR, &CHANNELS, state, wheels);
            if !out.standby_high {
                stby.iter_mut().for_each(|p| p.set_low());
            }
            let b = out.bridges;
            pwm_1a.set_timestamp(b[0].duty_pm);
            pwm_1b.set_timestamp(b[1].duty_pm);
            pwm_2a.set_timestamp(b[2].duty_pm);
            pwm_2b.set_timestamp(b[3].duty_pm);
            for ((in1, in2), bridge) in dirs.iter_mut().zip(b) {
                in1.set_level(Level::from(bridge.in1));
                in2.set_level(Level::from(bridge.in2));
            }
            if out.standby_high {
                stby.iter_mut().for_each(|p| p.set_high());
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
