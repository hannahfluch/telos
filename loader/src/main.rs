#![no_main]
#![no_std]

use uefi::{entry, Status};

extern crate alloc;

#[entry]
fn main() -> Status {
    todo!("loader here :)")
}
