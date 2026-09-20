#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::NonNull};

use bootinfo::BootInfo;
use framebuf::logger::Logger;
use log::info;

/// Enter the kernel with boot information supplied by the loader.
///
/// # Safety
/// The loader must provide an aligned, initialized, writable `BootInfo` and
/// exclusive access to its writer. The boot information, framebuffer, and font
/// must remain mapped and reserved while in use. Boot services must be exited,
/// interrupts disabled, and the stack prepared for the x86-64 SysV calling ABI.
// SAFETY: `_start` is the unique kernel entry symbol expected by the linker.
#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn _start(mut boot_info: NonNull<BootInfo>) -> ! {
    // SAFETY: the entry contract guarantees valid storage and exclusive access
    // while taking the writer. The loader no longer uses it after the handoff.
    let writer = unsafe { boot_info.as_mut() }.writer.take();
    if let Some(writer) = writer {
        Logger::init(writer);
        info!("Hello from kernel! :)");
    }

    // There is nowhere to return to until the kernel implements shutdown.
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // A panic may occur before the logger is ready; stop without relying on it.
    loop {
        core::hint::spin_loop();
    }
}
