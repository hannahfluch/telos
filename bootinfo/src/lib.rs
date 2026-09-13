#![no_std]

//! Shared Rust boot protocol for the Telos loader and kernel.

use framebuf::raw::write::RawWriter;

pub mod memory;

pub use memory::{MemoryDescriptor, MemoryMap, MemoryType};

/// Information handed to the kernel by the loader, necessary for boot.
///
/// The boot information, descriptor storage, and writer's framebuffer and font
/// must remain accessible after exiting boot services. Their backing memory must
/// not be reclaimed while in use. The loader must stop using the writer before
/// transferring it to the kernel.
#[derive(Debug)]
pub struct BootInfo {
    pub memory_map: MemoryMap,
    pub writer: Option<RawWriter>,
}
