#![no_std]

//! Shared Rust boot protocol for the Telos loader and kernel.

use framebuf::raw::write::RawWriter;

/// Information handed to the kernel by the loader, necessary for boot.
///
/// The boot information and writer's framebuffer and font
/// must remain accessible after exiting boot services. Their backing memory must
/// not be reclaimed while in use. The loader must stop using the writer before
/// transferring it to the kernel.
#[derive(Debug)]
pub struct BootInfo {
    pub writer: Option<RawWriter>,
}
