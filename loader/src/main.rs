#![no_main]
#![no_std]

use bootinfo::BootInfo;
use framebuf::{color, logger::Logger, raw::write::RawWriter};
use log::{debug, info};
use uefi::{
    boot::{self, PAGE_SIZE},
    entry, Status,
};

use crate::file::elf::Elf;

extern crate alloc;

mod error;
mod file;
mod graphics;
mod mem;

const PSF_FILE_NAME: &str = "font.psf";
const KERNEL_FILE_NAME: &str = "kernel.elf";

const KERNEL_STACK_SIZE: usize = 16 * 1024; // 16 KiB

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

    let stack = mem::allocate_kernel_stack(KERNEL_STACK_SIZE).unwrap();
    debug!(
        "Kernel stack: {:#x}..{:#x}, number of pages: {:#x}",
        stack.bottom(),
        stack.top(),
        stack.num_pages()
    );

    // Allocate a page for bootinfo (must stay accessible after exiting boot services)
    let boot_info = mem::allocate_bootinfo().unwrap();
    debug!("Boot info: {boot_info:?}");

    debug!("Exiting uefi boot services...");

    // Exit boot services (heap allocation etc. no longer possible)
    // note: because of the scope of this guide, we disregard the memory map
    _ = unsafe { boot::exit_boot_services(None) };

    debug!("Successfully exited boot services.");
    debug!("Constructing boot info and jumping to kernel...");

    // SAFETY: boot_info points to a suitably aligned, writable page allocation
    // large enough for BootInfo. It remains allocated after exiting boot services.
    unsafe {
        boot_info.as_ptr().write(BootInfo {
            writer: Logger::take_writer(),
        });
    }

    // Switching stacks, and passing boot_info come next.
    loop {
        core::hint::spin_loop();
    }
}
