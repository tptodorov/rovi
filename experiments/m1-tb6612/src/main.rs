#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Level, Output, OutputConfig};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{channel, timer, Ledc, LowSpeed, LSGlobalClkSource};
use esp_hal::main;
use esp_hal::time::Rate;
use esp_println::println;
use tb6612fng::{DriveCommand, Motor};

/// Pin map (see WIRING.md):
/// STBY=GPIO4, AIN1=GPIO5, AIN2=GPIO6, PWMA=GPIO7
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let ain1 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let ain2 = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let mut stby = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    // Wokwi models LEDC on ESP32-S3, while MCPWM is not yet simulated.
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut pwm_timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    pwm_timer
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(20),
        })
        .unwrap();

    let mut pwma = ledc.channel(channel::Number::Channel0, peripherals.GPIO7);
    pwma.configure(channel::config::Config {
        timer: &pwm_timer,
        duty_pct: 0,
        drive_mode: DriveMode::PushPull,
    })
    .unwrap();

    let mut motor = Motor::new(ain1, ain2, pwma).unwrap();
    stby.set_high();

    let delay = Delay::new();
    // Duty is crate-specific (0..=100 style in tb6612fng examples).
    const DUTY: u8 = 40;
    #[cfg(feature = "wokwi")]
    const DRIVE_MS: u32 = 100;
    #[cfg(not(feature = "wokwi"))]
    const DRIVE_MS: u32 = 2000;
    #[cfg(feature = "wokwi")]
    const STOP_MS: u32 = 50;
    #[cfg(not(feature = "wokwi"))]
    const STOP_MS: u32 = 1000;

    println!("rovi M1: TB6612FNG bring-up starting");

    loop {
        println!("forward");
        motor.drive(DriveCommand::Forward(DUTY)).ok();
        delay.delay_millis(DRIVE_MS);

        println!("stop");
        motor.drive(DriveCommand::Stop).ok();
        delay.delay_millis(STOP_MS);

        println!("reverse");
        motor.drive(DriveCommand::Backward(DUTY)).ok();
        delay.delay_millis(DRIVE_MS);

        println!("stop");
        motor.drive(DriveCommand::Stop).ok();
        delay.delay_millis(STOP_MS);
    }
}
