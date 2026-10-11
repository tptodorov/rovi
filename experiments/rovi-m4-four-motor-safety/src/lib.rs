//! M4 core: everything that decides what the motors do, host-tested with no board.
//! The firmware shell (`firmware/`) only moves bytes between this core and the pins.
#![no_std]

#[cfg(test)]
extern crate std;

pub mod command;
pub mod events;
pub mod motor;
pub mod power;
