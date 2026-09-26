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
use esp_hal::time::Rate;
use esp_println::println;
use tb6612fng::{DriveCommand, Motor};

/// Pin map (see docs/m1-wiring.md):
/// STBY=GPIO4, AIN1=GPIO5, AIN2=GPIO6, PWMA=GPIO7
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let ain1 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let ain2 = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let mut stby = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

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
    const DUTY: u8 = 40;

    println!("rovi M1: TB6612FNG bring-up starting");

    loop {
        println!("forward");
        motor.drive(DriveCommand::Forward(DUTY)).ok();
        delay.delay_millis(2000);

        println!("stop");
        motor.drive(DriveCommand::Stop).ok();
        delay.delay_millis(1000);

        println!("reverse");
        motor.drive(DriveCommand::Backward(DUTY)).ok();
        delay.delay_millis(2000);

        println!("stop");
        motor.drive(DriveCommand::Stop).ok();
        delay.delay_millis(1000);
    }
}
