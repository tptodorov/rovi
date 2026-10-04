#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write as _};
use embassy_executor::Spawner;
use embassy_futures::{
    join::join4,
    select::{select, Either},
};
use embassy_net::{
    tcp::TcpSocket, Config as NetConfig, Ipv4Address, Ipv4Cidr, Stack, StackResources,
    StaticConfigV4,
};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, ram, rmt::Rmt, rng::Rng, time::Rate, timer::timg::TimerGroup};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use esp_radio::wifi::{self, ap::AccessPointConfig, AuthenticationMethodConfig, WifiController};
use heapless::String;
use rovi_m3_wifi_queue::*;
use smart_leds::{SmartLedsWrite, RGB8};
use trouble_host::prelude::*;

esp_bootloader_esp_idf::esp_app_desc!();
static INGRESS: Mutex<CriticalSectionRawMutex, RefCell<Ingress>> =
    Mutex::new(RefCell::new(Ingress::new(1000, 250, 0)));
fn with<R>(f: impl FnOnce(&mut Ingress) -> R) -> R {
    INGRESS.lock(|q| f(&mut q.borrow_mut()))
}
fn now() -> u64 {
    Instant::now().as_millis()
}
fn setting(value: Option<&str>, default: u64) -> u64 {
    let n = value
        .map(|s| s.parse().expect("numeric timing setting"))
        .unwrap_or(default);
    assert!(n > 0 && n <= 60_000, "timing must be 1..60000 ms");
    n
}
fn submit(input: Input) -> (Status, u64, u64, usize) {
    let received = now();
    let (status, entry, depth) = with(|q| {
        let (status, entry) = q.submit(input, received);
        (status, entry, q.depth())
    });
    let order = entry.map_or(0, |e| e.order);
    println!("RX src={:?} session={} seq={} cmd={:?} rx_ms={} expires_ms={} order={} depth={} status={:?}",
        input.source,input.session,input.seq,input.command,received,input.expires_ms,order,depth,status);
    (status, order, received, depth)
}
async fn lease_ended(source: Source, session: u64) {
    loop {
        Timer::after_millis(10).await;
        if !with(|q| q.live(source, session)) {
            return;
        }
    }
}

#[gatt_server]
struct RoviServer {
    drive: DriveService,
}
#[gatt_service(uuid = "d47a0001-45b2-4d19-8db0-6bd87e9c0001")]
struct DriveService {
    #[characteristic(uuid = "d47a0002-45b2-4d19-8db0-6bd87e9c0001", write, value = 0)]
    command: u8,
}

async fn tcp(stack: Stack<'_>) -> ! {
    let mut rx = [0; 1024];
    let mut tx = [0; 1024];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
        socket.set_timeout(Some(Duration::from_secs(2)));
        if socket.accept(7777).await.is_err() {
            Timer::after_millis(10).await;
            continue;
        }
        let session = with(|q| q.connect(Source::Wifi, now()));
        println!("CONNECT src=Wifi session={} at_ms={}", session, now());
        let _ = select(lease_ended(Source::Wifi, session), async {
            let mut line = String::<256>::new();
            let (watchdog, age) = with(|q| (q.watchdog_ms, q.max_age_ms));
            writeln!(
                line,
                "ROVI version={} session={} now_ms={} max_age_ms={} watchdog_ms={} commands={}",
                VERSION,
                session,
                now(),
                age,
                watchdog,
                COMMANDS
            )
            .unwrap();
            socket.write_all(line.as_bytes()).await.ok()?;
            loop {
                let mut frame = [0; FRAME_LEN];
                socket.read_exact(&mut frame).await.ok()?;
                let Some(input) = decode(&frame) else {
                    println!("INVALID src=Wifi at_ms={}", now());
                    socket.write_all(b"ERR malformed\n").await.ok()?;
                    Timer::after_millis(1).await;
                    continue;
                };
                let (status, order, received, depth) = submit(input);
                line.clear();
                writeln!(
                    line,
                    "ACK seq={} status={:?} order={} rx_ms={} depth={}",
                    input.seq, status, order, received, depth
                )
                .unwrap();
                socket.write_all(line.as_bytes()).await.ok()?;
                // Bound ingress work per poll even when the TCP buffer stays readable.
                Timer::after_millis(1).await;
            }
            #[allow(unreachable_code)]
            Some(())
        })
        .await;
        with(|q| q.disconnect(Source::Wifi, now()));
        socket.abort();
        println!("DISCONNECT src=Wifi session={} at_ms={}", session, now());
        Timer::after_millis(10).await;
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    let timers = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timers.timer0, p.FROM_CPU_INTR0);
    let watchdog = setting(option_env!("ROVI_WATCHDOG_MS"), 1000);
    let age = setting(option_env!("ROVI_MAX_AGE_MS"), 250);
    let drain_ms = setting(option_env!("ROVI_CONSUMER_MS"), 20);
    let seed = Rng::new().random() as u64;
    with(|q| *q = Ingress::new(watchdog, age, seed << 16));
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
            .with_ssid("Rovi M3".try_into().unwrap())
            .with_channel(1)
            .with_max_connections(1)
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                password.try_into().unwrap(),
            )),
    ))
    .unwrap();
    let mut events = wifi.subscribe().unwrap();
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

    let connector =
        esp_radio::ble::controller::BleConnector::new(p.BT, Default::default()).unwrap();
    let controller = ExternalController::<_, 20>::new(connector);
    let mut ble_resources: HostResources<DefaultPacketPool, 1, 1> = HostResources::new();
    let stack = trouble_host::new(controller, &mut ble_resources)
        .set_random_address(Address::random([0xff, 0x9f, 0x1a, 0x05, 0xe4, 0xff]))
        .build();
    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();
    let server = RoviServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Rovi M3",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();
    println!("M3 board-only AP=Rovi M3 ip=192.168.4.1:7777 WPA2 channel=1 watchdog_ms={} max_age_ms={} consumer_ms={} capacity={}",watchdog,age,drain_ms,CAPACITY);

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
            &[AdStructure::CompleteLocalName(b"Rovi M3")],
            &mut scan_data,
        )
        .unwrap();
        loop {
            println!("BLE advertising at_ms={}", now());
            let advertiser = peripheral
                .advertise(
                    &Default::default(),
                    Advertisement::ConnectableScannableUndirected {
                        adv_data: &adv_data[..adv_len],
                        scan_data: &scan_data[..scan_len],
                    },
                )
                .await
                .unwrap();
            let connection = advertiser
                .accept()
                .await
                .unwrap()
                .with_attribute_server(&server)
                .unwrap();
            let session = with(|q| q.connect(Source::Ble, now()));
            let mut seq = 0u32;
            println!("CONNECT src=Ble session={} at_ms={}", session, now());
            loop {
                if !with(|q| q.live(Source::Ble, session)) {
                    connection.raw().disconnect();
                    break;
                }
                match select(lease_ended(Source::Ble, session), connection.next()).await {
                    Either::First(()) => {
                        connection.raw().disconnect();
                        break;
                    }
                    Either::Second(GattConnectionEvent::Disconnected { .. }) => break,
                    Either::Second(GattConnectionEvent::Gatt {
                        event: GattEvent::Write(event),
                    }) if event.handle() == server.drive.command.handle => {
                        let command = event.with_data(|offset, bytes| {
                            if offset == 0 && bytes.len() == 1 {
                                Command::parse(bytes[0])
                            } else {
                                None
                            }
                        });
                        let accepted = if let Some(command) = command {
                            seq = match seq.checked_add(1) {
                                Some(n) => n,
                                None => {
                                    connection.raw().disconnect();
                                    break;
                                }
                            };
                            let expires_ms = now() + age;
                            submit(Input {
                                source: Source::Ble,
                                session,
                                seq,
                                expires_ms,
                                command,
                            })
                            .0
                            .accepted()
                        } else {
                            println!("INVALID src=Ble at_ms={}", now());
                            false
                        };
                        if accepted {
                            event.accept().unwrap().send().await;
                        } else {
                            event
                                .reject(AttErrorCode::VALUE_NOT_ALLOWED)
                                .unwrap()
                                .send()
                                .await;
                        }
                        Timer::after_millis(1).await;
                    }
                    _ => {}
                }
            }
            with(|q| q.disconnect(Source::Ble, now()));
            println!("DISCONNECT src=Ble session={} at_ms={}", session, now());
        }
    };
    let consumer = async {
        let mut next_drain = now();
        loop {
            Timer::after_millis(10).await;
            let at = now();
            let drain = at >= next_drain;
            if drain {
                next_drain = at + drain_ms;
            }
            if let Some(output) = with(|q| q.poll(at, drain)) {
                let color = match output {
                    Output::Apply(e) => Some(match e.input.command {
                        Command::Forward => RGB8 { r: 0, g: 24, b: 0 },
                        Command::Reverse => RGB8 { r: 24, g: 0, b: 0 },
                        _ => RGB8 { r: 0, g: 0, b: 0 },
                    }),
                    Output::Safety { .. } => Some(RGB8 { r: 0, g: 0, b: 0 }),
                    Output::Stale(_) => None,
                };
                // Only this consumer touches the indicator. No motor GPIO is configured.
                if let Some(color) = color {
                    led.write([color]).unwrap();
                }
                println!(
                    "OUT at_ms={} depth={} event={:?}",
                    at,
                    with(|q| q.depth()),
                    output
                );
            }
        }
    };
    let departures = async {
        loop {
            use wifi::event::{EventInfo, MessageResult};
            let event = events.next_event().await;
            if matches!(
                event,
                MessageResult::Lagged(_)
                    | MessageResult::Message(EventInfo::AccessPointStationDisconnected { .. })
            ) {
                with(|q| q.disconnect(Source::Wifi, now()));
                println!("WIFI link departure/event loss at_ms={}", now());
            }
        }
    };
    let _ = select(
        join4(
            net_runner.run(),
            tcp(net),
            select(runner.run(), ble),
            consumer,
        ),
        departures,
    )
    .await;
    panic!("radio runner exited");
}
