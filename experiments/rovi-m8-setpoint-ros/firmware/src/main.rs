//! Firmware entry. The firmware only builds for the ESP32-S3 (see `.cargo/config.toml`).
#![cfg_attr(target_arch = "xtensa", no_std)]
#![cfg_attr(target_arch = "xtensa", no_main)]

#[cfg(target_arch = "xtensa")]
mod fw;

#[cfg(not(target_arch = "xtensa"))]
fn main() {
    eprintln!("build the firmware with --target xtensa-esp32s3-none-elf (see README)");
}
