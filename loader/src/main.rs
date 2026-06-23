#![no_main]
#![no_std]

use framebuf::{color, logger::Logger, raw::write::RawWriter};
use log::{debug, info};
use uefi::{boot::PAGE_SIZE, entry, Status};

extern crate alloc;

mod error;
mod file;
mod graphics;

const PSF_FILE_NAME: &str = "font.psf";

#[entry]
fn main() -> Status {
    let fb = graphics::initialize_framebuffer().unwrap(); // todo: maybe handle this differnetly
    fb.fill(color::BACKGROUND);

    let fb_addr = fb.ptr() as *mut u8 as u64;
    let fb_page_num = fb.ptr().len().div_ceil(PAGE_SIZE);

    let font = graphics::parse_psf_font(PSF_FILE_NAME).unwrap();

    let writer = RawWriter::new(font, fb, color::INFO, color::BACKGROUND);

    Logger::init(writer);

    info!("Hello World!!");
    debug!("Framebuffer: address: {fb_addr:#x}, number of pages: {fb_page_num:#x}");

    loop {}
}
