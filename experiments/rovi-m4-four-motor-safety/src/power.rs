//! INA226 rail monitor (TI datasheet SBOS547): register decoding and a min/max window.
//! The power-on configuration (0x4127: continuous shunt and bus, 1.1 ms each, no averaging) is
//! used as is, so the firmware only reads.

pub const ADDR: u8 = 0x40; // A0 = A1 = GND
pub const REG_SHUNT: u8 = 0x01;
pub const REG_BUS: u8 = 0x02;
pub const REG_MANUFACTURER: u8 = 0xFE;
pub const MANUFACTURER_ID: u16 = 0x5449;

/// Shunt voltage register (big-endian, two's complement) in µV: 2.5 µV per LSB.
pub fn shunt_uv(raw: [u8; 2]) -> i32 {
    i32::from(i16::from_be_bytes(raw)) * 5 / 2
}

/// Bus voltage register in mV: 1.25 mV per LSB.
pub fn bus_mv(raw: [u8; 2]) -> u32 {
    u32::from(u16::from_be_bytes(raw)) * 5 / 4
}

/// µV across a shunt of `milliohm` is mA.
pub fn current_ma(shunt_uv: i32, milliohm: u32) -> i32 {
    shunt_uv / milliohm as i32
}

/// One log line's worth of samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Power {
    pub bus_mv: u32,
    pub bus_min_mv: u32,
    /// Mean over the window.
    pub ma: i32,
    pub ma_max: i32,
}

#[derive(Default)]
pub struct Window {
    n: i32,
    sum_ma: i32,
    ma_max: i32,
    bus_mv: u32,
    bus_min_mv: u32,
}

impl Window {
    pub fn add(&mut self, bus_mv: u32, ma: i32) {
        if self.n == 0 {
            self.ma_max = ma;
            self.bus_min_mv = bus_mv;
        }
        self.n += 1;
        self.sum_ma += ma;
        self.ma_max = self.ma_max.max(ma);
        self.bus_mv = bus_mv;
        self.bus_min_mv = self.bus_min_mv.min(bus_mv);
    }

    /// The window so far, and a fresh one. `None` if it is empty.
    pub fn take(&mut self) -> Option<Power> {
        let w = core::mem::take(self);
        (w.n > 0).then(|| Power {
            bus_mv: w.bus_mv,
            bus_min_mv: w.bus_min_mv,
            ma: w.sum_ma / w.n,
            ma_max: w.ma_max,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_datasheet_example() {
        // SBOS547 example: shunt 1F40h = 8000 -> 20 mV, bus 2570h = 9584 -> 11.98 V.
        assert_eq!(shunt_uv([0x1F, 0x40]), 20_000);
        assert_eq!(bus_mv([0x25, 0x70]), 11_980);
    }

    #[test]
    fn negative_shunt_and_full_scale() {
        assert_eq!(shunt_uv([0xFF, 0xFF]), -2); // -1 LSB = -2.5 µV, truncated towards zero
        assert_eq!(shunt_uv([0x7F, 0xFF]), 81_917);
        assert_eq!(shunt_uv([0x80, 0x00]), -81_920);
    }

    #[test]
    fn current_from_shunt() {
        assert_eq!(current_ma(81_920, 100), 819); // R100 module: about 0.82 A full scale
        assert_eq!(current_ma(81_920, 10), 8_192); // R010 shunt
        assert_eq!(current_ma(-5_000, 10), -500);
    }

    #[test]
    fn window_keeps_mean_max_and_min_bus() {
        let mut w = Window::default();
        assert_eq!(w.take(), None);
        w.add(5000, 100);
        w.add(4700, 900);
        w.add(4950, 200);
        assert_eq!(
            w.take(),
            Some(Power {
                bus_mv: 4950,
                bus_min_mv: 4700,
                ma: 400,
                ma_max: 900
            })
        );
        assert_eq!(w.take(), None);
        w.add(5100, -50);
        assert_eq!(
            w.take().map(|p| (p.bus_min_mv, p.ma_max)),
            Some((5100, -50))
        );
    }
}
