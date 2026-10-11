//! M4 bench firmware shell: pins, MCPWM, INA226, serial bench commands, the 10 ms control tick.
//! Every decision is in the host-tested core; this file only moves values between it and the pins.
//! DRAFT: cross-built and linted, never flashed. Pin map: ../WIRING.md.

use embassy_futures::join::join3;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    rmt::Rmt,
    rtc_cntl::{Rtc, RwdtStage, RwdtStageAction},
    system::reset_reason,
    time::Rate,
    timer::timg::TimerGroup,
    uart::{Config as UartConfig, UartRx},
    Async,
};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds};
use rovi_m4_four_motor_safety::command::{self, Input as Cmd, Level as HangLevel};
use rovi_m4_four_motor_safety::events::{Event, Fault, Op, Reporter};
use rovi_m4_four_motor_safety::motor::{Config, Core, Drive, Mode, Output as Pins, WHEELS};
use rovi_m4_four_motor_safety::power::{self, Window};
use smart_leds::{SmartLedsWrite, RGB8};

esp_bootloader_esp_idf::esp_app_desc!();

#[path = "logger.rs"]
mod logger;

/// Control tick. The heartbeat toggles once per tick (a rising edge every 2 ticks).
const TICK_MS: u64 = 10;
/// PLACEHOLDER: the INA226 module's shunt (R100 = 100 mΩ, R010 = 10 mΩ). Read it off the module.
const SHUNT_MILLIOHM: u32 = 100;
const POWER_LOG_MS: u64 = 100;
/// PLACEHOLDER: the RTC watchdog resets the chip if the tick stops feeding it for this long.
const RWDT_MS: u64 = 1000;
/// MCPWM period in counts: duty in permille is the compare value (20 kHz, as M1).
const PWM_PERIOD: u16 = 999;

static INPUTS: Channel<CriticalSectionRawMutex, Cmd, 8> = Channel::new();

fn reset_name() -> &'static str {
    use esp_hal::rtc_cntl::SocResetReason as R;
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

/// One TB6612FNG channel's direction pins and PWM compare value.
struct Bridge<'d> {
    in1: Output<'d>,
    in2: Output<'d>,
}

impl Bridge<'_> {
    /// Returns the duty to write. Brake: IN1 = IN2 = H; coast and standby: IN1 = IN2 = L, PWM 0.
    fn set(&mut self, drive: Drive) -> u16 {
        let (a, b, duty) = match drive {
            Drive::Forward(d) => (true, false, d),
            Drive::Reverse(d) => (false, true, d),
            Drive::Brake => (true, true, 0),
            Drive::Coast => (false, false, 0),
        };
        self.in1.set_level(a.into());
        self.in2.set_level(b.into());
        duty
    }
}

fn color(mode: Mode) -> RGB8 {
    match mode {
        Mode::Standby(_) => RGB8 { r: 0, g: 0, b: 16 },
        Mode::Enabled => RGB8 { r: 0, g: 16, b: 0 },
    }
}

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    // Standby first. Until here the GPIOs float and the external pull-downs hold STBY_EN, the
    // heartbeat and STBY low (WIRING.md); now the firmware holds them low too.
    let mut stby_en = Output::new(p.GPIO4, Level::Low, OutputConfig::default());
    let mut heartbeat = Output::new(p.GPIO14, Level::Low, OutputConfig::default());
    let low = |pin: AnyPin<'static>| Output::new(pin, Level::Low, OutputConfig::default());
    // Driver channel order: D1A, D1B, D2A, D2B (motor::Channel::index).
    let mut bridges = [
        Bridge {
            in1: low(p.GPIO5.into()),
            in2: low(p.GPIO6.into()),
        },
        Bridge {
            in1: low(p.GPIO16.into()),
            in2: low(p.GPIO17.into()),
        },
        Bridge {
            in1: low(p.GPIO9.into()),
            in2: low(p.GPIO10.into()),
        },
        Bridge {
            in1: low(p.GPIO12.into()),
            in2: low(p.GPIO13.into()),
        },
    ];
    // Inputs only, so they never drive: STBY readback and INA226 ALERT are probed by the logic
    // analyzer; the draft does not act on them yet.
    let _stby_sense = Input::new(p.GPIO39, InputConfig::default());
    let _ina_alert = Input::new(p.GPIO38, InputConfig::default().with_pull(Pull::Up));

    let timers = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timers.timer0, p.FROM_CPU_INTR0);
    logger::emit(Event::Boot {
        reset: reset_name(),
    });

    let clock = PeripheralClockConfig::with_frequency(Rate::from_mhz(40)).unwrap();
    let mut mcpwm = McPwm::new(p.MCPWM0, clock);
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    mcpwm.operator1.set_timer(&mcpwm.timer0);
    let (d1a, d1b) = mcpwm.operator0.with_pins(
        p.GPIO7,
        PwmPinConfig::UP_ACTIVE_HIGH,
        p.GPIO15,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    let (d2a, d2b) = mcpwm.operator1.with_pins(
        p.GPIO8,
        PwmPinConfig::UP_ACTIVE_HIGH,
        p.GPIO11,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );
    let mut pwm = (d1a, d1b, d2a, d2b);
    pwm.0.set_timestamp(0);
    pwm.1.set_timestamp(0);
    pwm.2.set_timestamp(0);
    pwm.3.set_timestamp(0);
    let timer_clock = clock
        .timer_clock_with_frequency(PWM_PERIOD, PwmWorkingMode::Increase, Rate::from_khz(20))
        .unwrap();
    mcpwm.timer0.start(timer_clock);

    let mut i2c = I2c::new(
        p.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(p.GPIO1)
    .with_scl(p.GPIO2)
    .into_async();
    // UART0 RX (GPIO44, the CH343 port). esp-println keeps writing TX on the same UART.
    let rx = UartRx::new(p.UART0, UartConfig::default())
        .unwrap()
        .with_rx(p.GPIO44)
        .into_async();

    let rmt = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap();
    let mut led =
        RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb>::new_with_memsize(
            esp_hal_smartled::WS2812_TIMING,
            rmt.channel0,
            p.GPIO48,
            2,
            Rate::from_mhz(80),
        )
        .unwrap();

    let mut rtc = Rtc::new(p.RTC_TIMER);
    rtc.rwdt.set_timeout(
        RwdtStage::Stage0,
        esp_hal::time::Duration::from_millis(RWDT_MS),
    );
    rtc.rwdt
        .set_stage_action(RwdtStage::Stage0, RwdtStageAction::ResetSystem);
    rtc.rwdt.enable();

    let control = async {
        let mut core = Core::new(Config::BENCH);
        let mut reporter = Reporter::new(50);
        let mut window = Window::default();
        let mut ina_ok = ina_probe(&mut i2c).await;
        if !ina_ok {
            logger::emit(Event::Fault(Fault::Ina226));
        }
        let mut last_power_log = 0;
        let mut last_mode = None;
        let mut ticker = Ticker::every(Duration::from_millis(TICK_MS));
        loop {
            let now = Instant::now().as_millis();
            while let Ok(input) = INPUTS.try_receive() {
                match input {
                    Cmd::Enable => {
                        core.enable(now);
                        logger::emit(Event::Cmd(Op::Enable));
                    }
                    Cmd::Disable => {
                        core.disable();
                        logger::emit(Event::Cmd(Op::Disable));
                    }
                    Cmd::Wheels(w) => core.command(w, now),
                    Cmd::Zero(z) => {
                        core.set_zero(z);
                        logger::emit(Event::Cmd(Op::Zero(z)));
                    }
                    Cmd::Hang(level) => {
                        logger::emit(Event::Cmd(Op::Hang(level)));
                        Timer::after_millis(20).await; // let the line out first
                        heartbeat.set_level((level == HangLevel::High).into());
                        // Deliberate total hang: no tick, no heartbeat, no watchdog feed. The
                        // 74HC123 must drop STBY, then the RTC watchdog must reset the chip.
                        critical_section::with(|_| loop {
                            core::hint::spin_loop();
                        });
                    }
                }
            }

            if ina_ok {
                match ina_read(&mut i2c).await {
                    Some((bus_mv, ma)) => {
                        window.add(bus_mv, ma);
                        core.current(ma);
                    }
                    None => {
                        ina_ok = false;
                        logger::emit(Event::Fault(Fault::Ina226));
                    }
                }
            }

            let out = core.tick(now);
            let duty = apply(&out, &mut stby_en, &mut bridges);
            pwm.0.set_timestamp(duty[0]);
            pwm.1.set_timestamp(duty[1]);
            pwm.2.set_timestamp(duty[2]);
            pwm.3.set_timestamp(duty[3]);
            if out.enabled {
                stby_en.set_high();
            }

            let mode = core.mode();
            if let Some(ev) = reporter.report(mode, core.wheels(), core.target(), now) {
                logger::emit(ev);
            }
            if last_mode != Some(mode) {
                last_mode = Some(mode);
                let _ = led.write([color(mode)]);
            }
            if now.saturating_sub(last_power_log) >= POWER_LOG_MS {
                last_power_log = now;
                if let Some(p) = window.take() {
                    logger::emit(Event::Power(p));
                }
            }

            heartbeat.toggle();
            rtc.rwdt.feed();
            ticker.next().await;
        }
    };

    join3(logger::run(), serial(rx), control).await;
    unreachable!()
}

/// Sets the direction pins and returns the four duties. Standby: STBY_EN goes low before the
/// bridges and PWM change; enable: the caller raises it only after writing PWM.
fn apply(out: &Pins, stby_en: &mut Output, bridges: &mut [Bridge; WHEELS]) -> [u16; WHEELS] {
    if !out.enabled {
        stby_en.set_low();
    }
    let mut duty = [0u16; WHEELS];
    for ((b, d), drive) in bridges.iter_mut().zip(&mut duty).zip(out.channels) {
        *d = b.set(drive);
    }
    duty
}

async fn ina_reg(i2c: &mut I2c<'_, Async>, reg: u8) -> Option<[u8; 2]> {
    let mut b = [0; 2];
    i2c.write_read_async(power::ADDR, &[reg], &mut b)
        .await
        .ok()?;
    Some(b)
}

async fn ina_probe(i2c: &mut I2c<'_, Async>) -> bool {
    ina_reg(i2c, power::REG_MANUFACTURER)
        .await
        .map(u16::from_be_bytes)
        == Some(power::MANUFACTURER_ID)
}

/// Bus mV and rail mA from the power-on default configuration (continuous, 1.1 ms conversions).
async fn ina_read(i2c: &mut I2c<'_, Async>) -> Option<(u32, i32)> {
    let shunt = power::shunt_uv(ina_reg(i2c, power::REG_SHUNT).await?);
    let bus = power::bus_mv(ina_reg(i2c, power::REG_BUS).await?);
    Some((bus, power::current_ma(shunt, SHUNT_MILLIOHM)))
}

/// Reads bench command lines. A malformed or overlong line is logged as `op=Bad` and dropped.
async fn serial(mut rx: UartRx<'static, Async>) {
    let mut line = [0u8; 64];
    let mut len = 0;
    let mut overflow = false;
    let mut buf = [0u8; 32];
    loop {
        let Ok(n) = rx.read_async(&mut buf).await else {
            continue;
        };
        for &byte in &buf[..n] {
            if byte != b'\n' {
                if len < line.len() {
                    line[len] = byte;
                    len += 1;
                } else {
                    overflow = true;
                }
                continue;
            }
            let parsed = match core::str::from_utf8(&line[..len]) {
                Ok(s) if !overflow => command::parse(s),
                _ => Err(command::BadLine),
            };
            match parsed {
                Ok(Some(input)) => {
                    let _ = INPUTS.try_send(input); // full: dropped; the watchdog covers it
                }
                Ok(None) => {}
                Err(_) => logger::emit(Event::Cmd(Op::Bad)),
            }
            len = 0;
            overflow = false;
        }
    }
}
