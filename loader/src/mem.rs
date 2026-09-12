use crate::error::Result;
use uefi::boot::{self, AllocateType, MemoryType, PAGE_SIZE};

/// A page-backed, downward-growing kernel stack.
///
/// The allocation is intentionally retained for the kernel. After exiting boot
/// services, the kernel must keep these pages reserved while the stack is in use.
/// No guard page is installed; that requires setting up kernel page tables. (out of scope)
#[derive(Debug)]
pub(crate) struct KernelStack {
    /// Lowest address in the allocation (inclusive).
    bottom: usize,
    /// Highest address in the allocation (exclusive).
    /// Initial stack pointer before a call into the kernel.
    ///
    /// Page alignment also provides the 16-byte alignment required at an
    /// x86-64 call site. The handoff must still follow the kernel's entry ABI.
    top: usize,
    /// Actual number of pages allocated for the stack.
    num_pages: usize,
}

impl KernelStack {
    pub(crate) fn bottom(&self) -> usize {
        self.bottom
    }

    pub(crate) fn top(&self) -> usize {
        self.top
    }

    pub(crate) fn num_pages(&self) -> usize {
        self.num_pages
    }
}

/// Allocate at least `bytes` of stack space, rounded up to whole UEFI pages.
pub(crate) fn allocate_kernel_stack(bytes: usize) -> Result<KernelStack> {
    assert!(bytes > 0, "kernel stack must have a size bigger than 0!");
    let num_pages = bytes.div_ceil(PAGE_SIZE);
    let size = num_pages * PAGE_SIZE;
    let allocation =
        boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, num_pages)?;
    let bottom = allocation.as_ptr() as usize;
    let top = bottom + size;

    Ok(KernelStack {
        bottom,
        top,
        num_pages,
    })
}
