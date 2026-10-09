//! Log lines shared by the firmware and the sim car: `ev=<name> key=value ... at_ms=<n>`.
//! One format on both, so `sim/carlog.py` and the scenarios read a sim log and a board's serial log alike.

use crate::arbiter::{Effects, State};
use core::fmt::Arguments;

/// Calls `emit` once per event in `fx`.
pub fn effects(fx: Effects, at_ms: u64, mut emit: impl FnMut(Arguments)) {
    if let Some(s) = fx.claimed {
        emit(format_args!("ev=claimed source={s:?} at_ms={at_ms}"));
    }
    if let Some(r) = fx.stop {
        emit(format_args!("ev=stop reason={r:?} at_ms={at_ms}"));
    }
    if fx.released {
        emit(format_args!("ev=released at_ms={at_ms}"));
    }
}

/// Call when the state or the wheel commands (FL, FR, RR, RL) change.
pub fn state(state: State, w: [f32; 4], at_ms: u64, emit: impl FnOnce(Arguments)) {
    emit(format_args!(
        "ev=state state={state:?} wheels={},{},{},{} at_ms={at_ms}",
        w[0], w[1], w[2], w[3]
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arbiter::{Source, StopReason};
    use std::string::{String, ToString};
    use std::vec::Vec;

    #[test]
    fn effects_are_one_parsable_line_each() {
        let fx = Effects {
            claimed: Some(Source::Udp),
            released: true,
            stop: Some(StopReason::LeaseExpired),
        };
        let mut lines = Vec::<String>::new();
        effects(fx, 42, |a| lines.push(a.to_string()));
        assert_eq!(
            lines,
            [
                "ev=claimed source=Udp at_ms=42",
                "ev=stop reason=LeaseExpired at_ms=42",
                "ev=released at_ms=42"
            ]
        );
    }

    #[test]
    fn state_line_has_no_spaces_inside_values() {
        let mut line = String::new();
        state(State::Armed, [0.5, -0.5, 0.0, 1.0], 7, |a| {
            line = a.to_string()
        });
        assert_eq!(line, "ev=state state=Armed wheels=0.5,-0.5,0,1 at_ms=7");
        assert!(line.split(' ').all(|kv| kv.contains('=')));
    }
}
