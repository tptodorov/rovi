//! Where BLE, UDP and ROS deliberately differ, and why (design D8, transport variants). Everything
//! not listed here is the same on all three. `docs/reference/protocol.md` shows this table; a test
//! keeps it in step.

/// One intentional difference: an id, what differs, and why it stays.
pub struct Difference {
    pub id: &'static str,
    pub what: &'static str,
    pub why: &'static str,
}

pub const DIFFERENCES: &[Difference] = &[
    Difference {
        id: "T1",
        what: "Claim: BLE connects and claims on its first write; UDP sends `HELLO`; ROS claims on the first `cmd_vel` from a node in the graph",
        why: "Each transport's own session model (ADR-0013, ADR-0016)",
    },
    Difference {
        id: "T2",
        what: "Release: BLE disconnects; UDP sends `BYE`; ROS publishes `/rovi/release`",
        why: "Same as T1",
    },
    Difference {
        id: "T3",
        what: "ROS arms on its own stream; BLE and UDP need an explicit ARM",
        why: "Stock ROS tools publish only `cmd_vel` (ADR-0011)",
    },
    Difference {
        id: "T4",
        what: "Sequences: u32 serial on BLE and UDP; the ROS attachment's i64, per publisher",
        why: "ROS defines its attachment",
    },
    Difference {
        id: "T5",
        what: "No takeover notice on BLE",
        why: "BLE is never taken over",
    },
    Difference {
        id: "T6",
        what: "ROS Status text carries `session=`; binary Status does not",
        why: "ROS has no handshake that returns a session; bench tools use it",
    },
    Difference {
        id: "T7",
        what: "Echo: up to 32 bytes on BLE and UDP; `std_msgs/Int64` on ROS",
        why: "The ROS-native type",
    },
    Difference {
        id: "T8",
        what: "Mode command: BLE Control 3 / 4; UDP `MODE` and ROS `UInt8` 1 / 2",
        why: "Renumbering would break the accepted phone/firmware pairing for no gain; one `Command` type maps both",
    },
    Difference {
        id: "T9",
        what: "Status, Capabilities and network information are logfmt text on ROS, binary elsewhere",
        why: "Readable in ROS tools; both forms come from the same type",
    },
    Difference {
        id: "T11",
        what: "Delivery: BLE read/notify; UDP `INFO`/`HELLO_ACK` replies and unsolicited `STATUS`; ROS topics republished about once a second",
        why: "Each transport's way to deliver state; the conformance suite checks the values agree",
    },
];

/// The Markdown table for `docs/reference/protocol.md`.
pub fn table(out: &mut impl core::fmt::Write) -> core::fmt::Result {
    writeln!(out, "| # | Difference | Why it stays |")?;
    writeln!(out, "| --- | --- | --- |")?;
    for d in DIFFERENCES {
        writeln!(out, "| {} | {} | {} |", d.id, d.what, d.why)?;
    }
    Ok(())
}
