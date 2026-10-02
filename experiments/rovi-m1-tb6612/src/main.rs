#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::mcpwm::operator::PwmPinConfig;
use esp_hal::mcpwm::timer::PwmWorkingMode;
use esp_hal::mcpwm::{McPwm, PeripheralClockConfig};
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use tb6612fng::{DriveCommand, Motor};

esp_bootloader_esp_idf::esp_app_desc!();

/// Pin map (see WIRING.md):
/// STBY=GPIO4, AIN1=GPIO5, AIN2=GPIO6, PWMA=GPIO7
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

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
    const LED_OFF: RGB8 = RGB8 { r: 0, g: 0, b: 0 };
    const LED_FORWARD: RGB8 = RGB8 { r: 0, g: 24, b: 0 };
    const LED_REVERSE: RGB8 = RGB8 { r: 24, g: 0, b: 0 };
    const LED_STANDBY: RGB8 = RGB8 { r: 0, g: 0, b: 24 };
    led.write([LED_OFF]).unwrap();

    let ain1 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let ain2 = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let mut stby = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    // MCPWM0 is the ESP32-S3's dedicated motor-control PWM peripheral (as
    // opposed to LEDC, which is designed for LED dimming).
    let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();
    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    let pwma = mcpwm
        .operator0
        .with_pin_a(peripherals.GPIO7, PwmPinConfig::UP_ACTIVE_HIGH);

    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, Rate::from_khz(20))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);

    let mut motor = Motor::new(ain1, ain2, pwma).unwrap();
    stby.set_high();

    let delay = Delay::new();
    // Duty is crate-specific (0..=100 style in tb6612fng examples).
    const MAX_DUTY: u8 = 40;
    const RAMP_STEP: u8 = 5;
    const RAMP_INTERVAL_MS: u32 = 200;
    const STOP_MS: u32 = 1000;
    const STANDBY_TEST_MS: u32 = 1000;

    println!("rovi M1: TB6612FNG bring-up starting");

    let mut command = |motor_command, label: &str, color, duty: u8, standby: Option<bool>| {
        if let Some(enabled) = standby {
            if enabled {
                stby.set_high();
            } else {
                stby.set_low();
            }
        }

        if let Some(motor_command) = motor_command {
            if let Err(error) = motor.drive(motor_command) {
                println!("motor command failed ({}): {:?}", label, error);
                stby.set_low();
                led.write([LED_STANDBY]).unwrap();
                return false;
            }
        }

        led.write([color]).unwrap();
        if duty == 0 {
            println!("{}", label);
        } else {
            println!("{} {}%", label, duty);
        }
        true
    };

    if !command(
        Some(DriveCommand::Stop),
        "standby disabled",
        LED_STANDBY,
        0,
        Some(false),
    ) {
        loop {
            delay.delay_millis(1000);
        }
    }
    delay.delay_millis(STANDBY_TEST_MS);
    if !command(None, "standby enabled", LED_OFF, 0, Some(true)) {
        loop {
            delay.delay_millis(1000);
        }
    }

    loop {
        let mut duty = 0;
        while duty < MAX_DUTY {
            duty = (duty + RAMP_STEP).min(MAX_DUTY);
            if !command(
                Some(DriveCommand::Forward(duty)),
                "forward",
                LED_FORWARD,
                duty,
                None,
            ) {
                loop {
                    delay.delay_millis(1000);
                }
            }
            delay.delay_millis(RAMP_INTERVAL_MS);
        }

        if !command(
            Some(DriveCommand::Forward(MAX_DUTY)),
            "standby disabled",
            LED_STANDBY,
            MAX_DUTY,
            Some(false),
        ) {
            loop {
                delay.delay_millis(1000);
            }
        }
        delay.delay_millis(STANDBY_TEST_MS);
        if !command(None, "standby enabled", LED_FORWARD, MAX_DUTY, Some(true)) {
            loop {
                delay.delay_millis(1000);
            }
        }
        delay.delay_millis(STANDBY_TEST_MS);

        while duty > RAMP_STEP {
            duty -= RAMP_STEP;
            if !command(
                Some(DriveCommand::Forward(duty)),
                "forward",
                LED_FORWARD,
                duty,
                None,
            ) {
                loop {
                    delay.delay_millis(1000);
                }
            }
            delay.delay_millis(RAMP_INTERVAL_MS);
        }
        if !command(Some(DriveCommand::Stop), "stop", LED_OFF, 0, None) {
            loop {
                delay.delay_millis(1000);
            }
        }
        delay.delay_millis(STOP_MS);

        duty = 0;
        while duty < MAX_DUTY {
            duty = (duty + RAMP_STEP).min(MAX_DUTY);
            if !command(
                Some(DriveCommand::Backward(duty)),
                "reverse",
                LED_REVERSE,
                duty,
                None,
            ) {
                loop {
                    delay.delay_millis(1000);
                }
            }
            delay.delay_millis(RAMP_INTERVAL_MS);
        }

        while duty > RAMP_STEP {
            duty -= RAMP_STEP;
            if !command(
                Some(DriveCommand::Backward(duty)),
                "reverse",
                LED_REVERSE,
                duty,
                None,
            ) {
                loop {
                    delay.delay_millis(1000);
                }
            }
            delay.delay_millis(RAMP_INTERVAL_MS);
        }
        if !command(Some(DriveCommand::Stop), "stop", LED_OFF, 0, None) {
            loop {
                delay.delay_millis(1000);
            }
        }
        delay.delay_millis(STOP_MS);
    }
}
