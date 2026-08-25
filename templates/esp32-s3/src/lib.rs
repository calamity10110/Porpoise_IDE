//! lib.rs — Module declarations for the Porpoise ESP32-S3 template.

#![no_std]
#![no_main]

extern crate alloc;

pub mod board;
pub mod comms;
pub mod orchestration;
pub mod peripherals;
pub mod sensors;
