use crate::{
    error::{KernelLoadError, Result},
    file,
};
use alloc::slice;
use goblin::elf::{
    header::{EM_X86_64, ET_EXEC},
    program_header::{PF_X, PT_LOAD},
};
use uefi::boot::{self, MemoryType, PAGE_SIZE};

/// Loaded kernel ELF metadata.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Elf {
    /// Virtual entry-point address from the ELF header.
    entry_point: u64,
    /// Base address of the loaded kernel image.
    file_base: u64,
    /// Number of physical pages occupied by the loaded image.
    num_pages: usize,
}

impl Elf {
    /// Retrieve the virtual entry-point address.
    pub(crate) fn entry(&self) -> u64 {
        self.entry_point
    }

    /// Retrieve the physical load address.
    pub(crate) fn base(&self) -> u64 {
        self.file_base
    }

    /// Retrieve number of pages
    pub(crate) fn num_pages(&self) -> usize {
        self.num_pages
    }
}

impl Elf {
    /// Parse a 64-bit kernel ELF and load its segments into physical memory.
    pub(crate) fn load_kernel(filename: &'static str) -> Result<Elf> {
        let data = file::get_file_data(filename)?;
        let data = data.as_slice();

        let elf = goblin::elf::Elf::parse(data)?;
        let mut dest_start = u64::MAX;
        let mut dest_end = 0;
        let mut has_loadable_segment = false;
        let mut entry_is_executable = false;

        if !elf.is_64
            || !elf.little_endian
            || elf.header.e_machine != EM_X86_64
            || elf.header.e_type != ET_EXEC
        {
            return Err(KernelLoadError::InvalidFormat.into());
        }

        // Validate the load segments before allocating or copying anything.
        for pheader in &elf.program_headers {
            if pheader.p_type != PT_LOAD || pheader.p_memsz == 0 {
                continue;
            }

            if pheader.p_filesz > pheader.p_memsz {
                return Err(KernelLoadError::InvalidSegment.into());
            }

            let file_end = pheader
                .p_offset
                .checked_add(pheader.p_filesz)
                .ok_or(KernelLoadError::InvalidSegment)?;
            let memory_end = pheader
                .p_paddr
                .checked_add(pheader.p_memsz)
                .ok_or(KernelLoadError::InvalidSegment)?;
            let virtual_end = pheader
                .p_vaddr
                .checked_add(pheader.p_memsz)
                .ok_or(KernelLoadError::InvalidSegment)?;

            if file_end > data.len() as u64 {
                return Err(KernelLoadError::InvalidSegment.into());
            }

            has_loadable_segment = true;
            dest_start = dest_start.min(pheader.p_paddr);
            dest_end = dest_end.max(memory_end);

            if pheader.p_flags & PF_X != 0 && (pheader.p_vaddr..virtual_end).contains(&elf.entry) {
                entry_is_executable = true;
            }
        }

        if !has_loadable_segment {
            return Err(KernelLoadError::NoLoadableSegments.into());
        }
        if !entry_is_executable {
            return Err(KernelLoadError::InvalidEntryPoint.into());
        }

        // UEFI page allocations require a page-aligned physical range.
        let page_mask = PAGE_SIZE as u64 - 1;
        dest_start &= !page_mask;
        dest_end = dest_end
            .checked_add(page_mask)
            .ok_or(KernelLoadError::InvalidSegment)?
            & !page_mask;
        let num_pages = usize::try_from((dest_end - dest_start) / PAGE_SIZE as u64)
            .map_err(|_| KernelLoadError::InvalidSegment)?;

        // Allocate the physical range specified by the ELF load segments.
        let allocation = boot::allocate_pages(
            boot::AllocateType::Address(dest_start),
            MemoryType::LOADER_CODE,
            num_pages,
        )?;

        // Copy file-backed bytes and zero the remainder (for example, .bss).
        for pheader in &elf.program_headers {
            if pheader.p_type != PT_LOAD || pheader.p_memsz == 0 {
                continue;
            }

            // These conversions are safe after the range validation above.
            let offset = pheader.p_offset as usize;
            let size_in_file = pheader.p_filesz as usize;
            let size_in_memory = pheader.p_memsz as usize;

            // SAFETY: AllocatePages reserved the page-aligned range containing
            // this segment, and UEFI currently identity-maps physical memory.
            let dest =
                unsafe { slice::from_raw_parts_mut(pheader.p_paddr as *mut u8, size_in_memory) };
            dest[..size_in_file].copy_from_slice(&data[offset..offset + size_in_file]);
            dest[size_in_file..].fill(0);
        }

        Ok(Elf {
            entry_point: elf.entry,
            file_base: allocation.as_ptr() as u64,
            num_pages,
        })
    }
}
