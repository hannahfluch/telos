//! Firmware-independent description of physical memory after boot services exit.

/// A memory map stored in persistent, loader-allocated memory.
///
/// Descriptors must describe nonoverlapping, page-aligned ranges. The slice length
/// counts descriptors, not bytes. Its backing allocation must remain mapped and
/// reserved for the kernel's lifetime; a temporary UEFI allocation is not enough.
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub descriptors: &'static [MemoryDescriptor],
}

/// A physical range starting at `phys_start`, covering `num_pages` 4 KiB pages.
///
/// The exclusive end is `phys_start + num_pages * 4096`; the loader must ensure
/// this calculation does not overflow when constructing the map.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryDescriptor {
    pub phys_start: u64,
    pub num_pages: u64,
    pub r#type: MemoryType,
}

/// How the kernel may use a physical memory range.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Free RAM, including boot-services memory that is no longer needed.
    Available = 0,
    /// Unavailable or unknown memory, including runtime services, ACPI, and MMIO.
    Reserved = 1,
    /// The entire loaded kernel image, including its data and zero-filled regions.
    KernelCode = 2,
    /// The kernel's active stack; keep reserved while in use.
    KernelStack = 3,
    /// Boot information, memory-map descriptors, and font data.
    KernelData = 4,
    /// Loader code and data; reclaim only after the handoff and all uses end.
    Loader = 5,
}
