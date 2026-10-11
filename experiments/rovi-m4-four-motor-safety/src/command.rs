//! Bench command lines from the laptop (serial, one per line). Integers would hide the non-finite
//! case, so wheel commands are floats and `nan`/`inf` parse like any other value.
//!
//! ```text
//! en                  enable (leave standby)
//! stby                standby
//! w <fl> <fr> <rr> <rl>   wheel commands, -1.0..=1.0 (clamped), renew the watchdog
//! zero brake|coast    zero-command mode (O5)
//! hang high|low       stop the control loop with the heartbeat pin held high or low (fail-safe test)
//! ```

use crate::motor::{ZeroMode, WHEELS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    High,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Enable,
    Disable,
    Wheels([f32; WHEELS]),
    Zero(ZeroMode),
    Hang(Level),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadLine;

/// `Ok(None)` for a blank line.
pub fn parse(line: &str) -> Result<Option<Input>, BadLine> {
    let mut words = line.split_whitespace();
    let Some(cmd) = words.next() else {
        return Ok(None);
    };
    let input = match (cmd, words.next()) {
        ("en", None) => Input::Enable,
        ("stby", None) => Input::Disable,
        ("zero", Some("brake")) => Input::Zero(ZeroMode::Brake),
        ("zero", Some("coast")) => Input::Zero(ZeroMode::Coast),
        ("hang", Some("high")) => Input::Hang(Level::High),
        ("hang", Some("low")) => Input::Hang(Level::Low),
        ("w", Some(first)) => {
            let mut w = [0.0; WHEELS];
            w[0] = first.parse().map_err(|_| BadLine)?;
            for v in &mut w[1..] {
                *v = words.next().ok_or(BadLine)?.parse().map_err(|_| BadLine)?;
            }
            Input::Wheels(w)
        }
        _ => return Err(BadLine),
    };
    match words.next() {
        None => Ok(Some(input)),
        Some(_) => Err(BadLine),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_command() {
        assert_eq!(parse("en"), Ok(Some(Input::Enable)));
        assert_eq!(parse("stby\r"), Ok(Some(Input::Disable)));
        assert_eq!(
            parse("w 0.3 -0.25 1 0"),
            Ok(Some(Input::Wheels([0.3, -0.25, 1.0, 0.0])))
        );
        assert_eq!(parse("zero brake"), Ok(Some(Input::Zero(ZeroMode::Brake))));
        assert_eq!(parse("zero coast"), Ok(Some(Input::Zero(ZeroMode::Coast))));
        assert_eq!(parse("hang high"), Ok(Some(Input::Hang(Level::High))));
        assert_eq!(parse("  hang   low "), Ok(Some(Input::Hang(Level::Low))));
        assert_eq!(parse(""), Ok(None));
        assert_eq!(parse(" \r"), Ok(None));
    }

    #[test]
    fn non_finite_values_reach_the_core() {
        match parse("w nan 0 0 0") {
            Ok(Some(Input::Wheels(w))) => assert!(w[0].is_nan()),
            other => panic!("{other:?}"),
        }
        match parse("w 0 inf -inf 0") {
            Ok(Some(Input::Wheels(w))) => assert!(w[1].is_infinite() && w[2].is_infinite()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rejects_malformed_lines() {
        for bad in [
            "go",
            "w",
            "w 1 2 3",
            "w 1 2 3 4 5",
            "w 1 x 3 4",
            "zero",
            "zero hard",
            "hang",
            "hang up",
            "en now",
        ] {
            assert_eq!(parse(bad), Err(BadLine), "{bad}");
        }
    }
}
