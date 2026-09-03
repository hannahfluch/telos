use framebuf::{
    fonts::psf::{
        PSF1_MAGIC, PSF2_MAGIC, RawFont,
        header::{Header, PSF1Header, PSF2Header},
    },
    raw::RawFrameBuffer,
};
use uefi::{
    boot::{self, MemoryType, PAGE_SIZE},
    proto::console::gop,
};

use crate::{
    error::{PsfParseError, Result},
    file,
};

pub(crate) fn initialize_framebuffer() -> Result<RawFrameBuffer> {
    // GOP describes the firmware-selected display mode and its backing memory.
    let handle = boot::get_handle_for_protocol::<gop::GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<gop::GraphicsOutput>(handle)?;

    let gop_mode = gop.current_mode_info();
    let gop_fb_size = gop.frame_buffer().size();
    let (format, bpp) = match gop_mode.pixel_format() {
        gop::PixelFormat::Rgb => (framebuf::raw::PixelFormat::Rgb32bit, 4),
        gop::PixelFormat::Bgr => (framebuf::raw::PixelFormat::Bgr32bit, 4),
        gop::PixelFormat::Bitmask => unimplemented!(),
        gop::PixelFormat::BltOnly => unimplemented!(),
    };

    Ok(unsafe {
        // SAFETY: GOP supplies the pointer, length, and display-mode metadata.
        RawFrameBuffer::new(
            gop.frame_buffer().as_mut_ptr(),
            gop_fb_size,
            gop_mode.resolution().0,
            gop_mode.resolution().1,
            gop_mode.stride(),
            format,
            bpp,
        )
    })
}

/// Validate a PSF file and copy it into page-backed memory.
pub(crate) fn parse_psf_font(fontname: &'static str) -> Result<RawFont> {
    let font_data = file::get_file_data(fontname)?;
    let font_data_ptr = font_data.as_ptr();

    // A PSF1 header is the smallest supported header.
    if font_data.len() < size_of::<PSF1Header>() {
        return Err(PsfParseError::InsufficientDataForPSFHeader.into());
    }

    // SAFETY: The minimum-length check covers two bytes. `read_unaligned`
    // handles the fact that file bytes have no typed alignment guarantee.
    let magic = unsafe { (font_data_ptr as *const u16).read_unaligned() };

    // PSF1 and PSF2 use different header layouts and magic widths.
    if magic == PSF1_MAGIC {
        // SAFETY: The initial length check covers the complete PSF1 header.
        let header = unsafe { (font_data_ptr as *const PSF1Header).read_unaligned() };
        let glyph_buffer_length = if header.font_mode & 1 != 0 { 512 } else { 256 };
        let glyph_buffer_size = glyph_buffer_length * header.character_size as usize;

        let total_size = size_of::<PSF1Header>() + glyph_buffer_size;

        // Validate every source byte before performing raw-pointer copies.
        if font_data.len() < total_size {
            return Err(PsfParseError::InsufficientDataForPSF1.into());
        }

        let page_count = total_size.div_ceil(PAGE_SIZE);

        // The temporary Vec is allocator-owned. A page allocation can be tracked
        // explicitly and handed to the kernel with the other loader data.
        let font_address = boot::allocate_pages(
            boot::AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            page_count,
        )?;

        unsafe {
            // SAFETY: The length check covers the complete source range, while
            // allocate_pages returned at least total_size writable bytes.
            core::ptr::copy_nonoverlapping(font_data_ptr, font_address.as_ptr(), total_size);
        }

        // The glyphs follow the header in both the file and its persistent copy.
        let glyph_buffer_ptr = unsafe { (font_address.as_ptr()).add(size_of::<PSF1Header>()) };

        Ok(unsafe {
            // SAFETY: The new page allocation contains all glyph bytes and
            // remains allocated after the temporary file buffer is dropped.
            RawFont::new(
                Header::V1(header),
                glyph_buffer_ptr as *const u8,
                glyph_buffer_length,
            )
        })
    } else {
        // SAFETY: The initial length check covers four bytes, which is also the
        // width of the PSF2 magic value.
        let magic = unsafe { (font_data_ptr as *const u32).read_unaligned() };

        if magic != PSF2_MAGIC {
            return Err(PsfParseError::InvalidPSFMagic(magic).into());
        }

        if font_data.len() < size_of::<PSF2Header>() {
            return Err(PsfParseError::InsufficientDataForPSFHeader.into());
        }

        // SAFETY: The PSF2-specific length check covers the complete header.
        let header = unsafe { (font_data_ptr as *const PSF2Header).read_unaligned() };

        let glyph_buffer_size = (header.length * header.glyph_size) as usize;
        let total_size = size_of::<PSF2Header>() + glyph_buffer_size;

        // Validate every source byte before performing raw-pointer copies.
        if font_data.len() < total_size {
            return Err(PsfParseError::InsufficientDataForPSF2.into());
        }

        let page_count = total_size.div_ceil(PAGE_SIZE);

        // Preserve the font independently of the temporary file buffer.
        let font_address = boot::allocate_pages(
            boot::AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            page_count,
        )?;

        unsafe {
            // SAFETY: The length check covers the header, and allocate_pages
            // returned at least total_size writable bytes in a disjoint region.
            core::ptr::copy_nonoverlapping(
                font_data_ptr,
                font_address.as_ptr(),
                size_of::<PSF2Header>(),
            );
        }

        // Store the glyphs directly after the persistent header copy.
        let glyph_buffer_ptr = unsafe { (font_address.as_ptr()).add(size_of::<PSF2Header>()) };

        unsafe {
            // SAFETY: The total-size check covers the source glyph range. The
            // destination lies within the new allocation and cannot overlap it.
            core::ptr::copy_nonoverlapping(
                font_data_ptr.add(size_of::<PSF2Header>()),
                glyph_buffer_ptr,
                glyph_buffer_size,
            );
        }

        Ok(unsafe {
            // SAFETY: The new page allocation contains all glyph bytes and
            // remains allocated after the temporary file buffer is dropped.
            RawFont::new(
                Header::V2(header),
                glyph_buffer_ptr as *const u8,
                header.length as usize,
            )
        })
    }
}
