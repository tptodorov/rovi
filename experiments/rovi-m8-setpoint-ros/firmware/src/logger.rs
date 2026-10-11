//! The log sink (spec: docs/LOGGING.md). Emitters call `emit`, which copies a `Record` into a ring:
//! no formatting, no I/O, no waiting. `run` takes records off the ring, formats one line and writes
//! it with a single `write_bytes`, then sleeps for as long as the UART needs to send it. The UART
//! FIFO (128 bytes) therefore never fills and nothing spins, so logging cannot stall the 10 ms tick.

use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Instant, Timer};
use esp_println::Printer;
use rovi_m8_setpoint_ros::events::{Event, LineBuf, Record, MAX_LINE};

static RING: Channel<CriticalSectionRawMutex, Record, 32> = Channel::new();
static DROPPED: AtomicU32 = AtomicU32::new(0);

/// Microseconds per byte on UART0 at 115200 8N1. On the native USB port this only caps the rate.
const US_PER_BYTE: u64 = 87;

/// Queues one event. A full ring drops it and counts it (`ev=dropped`).
pub fn emit(ev: Event) {
    let at_ms = Instant::now().as_millis() as u32;
    if RING.try_send(Record { at_ms, ev }).is_err() {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

pub async fn run() {
    loop {
        let record = RING.receive().await;
        let mut line = LineBuf::<MAX_LINE>::new();
        let _ = writeln!(line, "{record}");
        Printer::write_bytes(line.as_bytes());
        let lost = DROPPED.swap(0, Ordering::Relaxed);
        if lost > 0 {
            emit(Event::Dropped(lost));
        }
        Timer::after_micros(line.as_bytes().len() as u64 * US_PER_BYTE).await;
    }
}
