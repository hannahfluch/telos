#![no_main]
#![no_std]

use framebuf::{color, logger::Logger, raw::write::RawWriter};
use log::{debug, info};
use uefi::{boot::PAGE_SIZE, entry, Status};

use crate::file::elf::Elf;

extern crate alloc;

mod error;
mod file;
mod graphics;

const PSF_FILE_NAME: &str = "font.psf";
const KERNEL_FILE_NAME: &str = "kernel.elf";

#[entry]
fn main() -> Status {
    // Panicking is sufficient during bring-up; later stages should display the
    // error through an available console and halt cleanly.
    let fb = graphics::initialize_framebuffer().unwrap();
    fb.fill(color::BACKGROUND);

    // Keep the complete framebuffer range available for future page tables.
    let fb_addr = fb.ptr() as *mut u8 as u64;
    let fb_page_num = fb.ptr().len().div_ceil(PAGE_SIZE);

    let font = graphics::parse_psf_font(PSF_FILE_NAME).unwrap();
    let writer = RawWriter::new(font, fb, color::INFO, color::BACKGROUND);
    Logger::init(writer);

    info!("Hello World!!");
    debug!("Framebuffer: address: {fb_addr:#x}, number of pages: {fb_page_num:#x}");

    // Load kernel into memory
    let kernel = Elf::load_kernel(KERNEL_FILE_NAME).unwrap();
    debug!(
        "Kernel entry point: {:#x}, number of pages: {:#x}",
        kernel.entry(),
        kernel.num_pages()
    );

    // Setting up a kernel stack is the next stage; stop here for this milestone.
    loop {
        core::hint::spin_loop();
    }
}
