#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::select;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::ram;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use trouble_host::prelude::*;

esp_bootloader_esp_idf::esp_app_desc!();

#[gatt_server]
struct RoviServer {
    drive: DriveService,
}

#[gatt_service(uuid = "d47a0001-45b2-4d19-8db0-6bd87e9c0001")]
struct DriveService {
    #[characteristic(uuid = "d47a0002-45b2-4d19-8db0-6bd87e9c0001", write, value = 0)]
    command: u8,
}

#[derive(Clone, Copy)]
enum DriveCommand {
    Stop,
    Forward,
    Reverse,
}

impl DriveCommand {
    fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Stop),
            1 => Some(Self::Forward),
            2 => Some(Self::Reverse),
            _ => None,
        }
    }

    fn led(self) -> RGB8 {
        match self {
            Self::Stop => RGB8 { r: 0, g: 0, b: 0 },
            Self::Forward => RGB8 { r: 0, g: 24, b: 0 },
            Self::Reverse => RGB8 { r: 24, g: 0, b: 0 },
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Forward => "forward",
            Self::Reverse => "reverse",
        }
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);

    let timers = TimerGroup::new(peripherals.TIMG0);
    // The scheduler must run before the BLE controller is initialized.
    esp_rtos::start(timers.timer0, peripherals.FROM_CPU_INTR0);

    let led_freq = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, led_freq).unwrap();
    let mut led =
        RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb>::new_with_memsize(
            esp_hal_smartled::WS2812_TIMING,
            rmt.channel0,
            peripherals.GPIO48,
            2,
            led_freq,
        )
        .unwrap();
    let advertising = RGB8 { r: 0, g: 0, b: 24 };
    led.write([advertising]).unwrap();

    let connector = esp_radio::ble::controller::BleConnector::new(
        peripherals.BT,
        esp_radio::ble::Config::default(),
    )
    .unwrap();
    let controller = ExternalController::<_, 20>::new(connector);
    let mut resources: HostResources<DefaultPacketPool, 1, 1> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(Address::random([0xff, 0x9f, 0x1a, 0x05, 0xe4, 0xff]));
    let stack = stack.build();
    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();

    let server = RoviServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Rovi M2",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();

    println!("Rovi M2 BLE peripheral started; advertising as Rovi M2");
    let _ = select(runner.run(), async {
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
            &[AdStructure::CompleteLocalName(b"Rovi M2")],
            &mut scan_data,
        )
        .unwrap();

        loop {
            println!("Advertising");
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
            let connection = advertiser.accept().await.unwrap();
            let connection = connection.with_attribute_server(&server).unwrap();
            println!("BLE client connected");

            loop {
                match connection.next().await {
                    GattConnectionEvent::Disconnected { reason } => {
                        println!("BLE client disconnected: {:?}", reason);
                        led.write([advertising]).unwrap();
                        break;
                    }
                    GattConnectionEvent::Gatt {
                        event: GattEvent::Write(event),
                    } if event.handle() == server.drive.command.handle => {
                        match event.value(&server.drive.command) {
                            Ok(value) => match DriveCommand::from_byte(value) {
                                Some(command) => {
                                    event.accept().unwrap().send().await;
                                    led.write([command.led()]).unwrap();
                                    println!(
                                        "Accepted drive command: {} ({})",
                                        command.name(),
                                        value
                                    );
                                }
                                None => {
                                    event
                                        .reject(AttErrorCode::VALUE_NOT_ALLOWED)
                                        .unwrap()
                                        .send()
                                        .await;
                                    println!("Rejected unknown drive command: {}", value);
                                }
                            },
                            Err(_) => {
                                event
                                    .reject(AttErrorCode::INVALID_ATTRIBUTE_VALUE_LENGTH)
                                    .unwrap()
                                    .send()
                                    .await;
                                println!("Rejected malformed drive command");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    })
    .await;

    loop {
        core::future::pending::<()>().await;
    }
}
