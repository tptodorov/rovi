//! Opt-in reversal harness, absent from normal builds. Only the fixed UDP probe's
//! vx=±0.1 markers on Geometry::CAR request full-scale vectors; everything else coasts.
//! The device still owns validation, arming, lease expiry and STOP.
use rovi_core::{arbiter::Source, device::Device, events};

pub const BANNER: &str = "BENCH ONLY motor-reversal max_drive_pm=700 markers_pm=83,-83";

pub fn wheels(dev: &Device, now: u64) -> [f32; 4] {
    if !dev
        .arbiter()
        .owner()
        .is_some_and(|o| o.source == Source::Udp)
    {
        return [0.0; 4];
    }
    // CAR normalises vx=0.1 to 83 permille on all wheels. No API limit changes.
    match events::permille(dev.wheels(now)) {
        [83, 83, 83, 83] => [1.0; 4],
        [-83, -83, -83, -83] => [-1.0; 4],
        _ => [0.0; 4],
    }
}
