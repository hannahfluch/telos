#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Do not mangle `_start`: the linker looks for this exact symbol name.
#[unsafe(no_mangle)]
pub extern "sysv64" fn _start() -> ! {
    // There is nowhere to return to until the kernel implements shutdown.
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Logging is not initialized yet, so the only safe response is to stop.
    loop {
        core::hint::spin_loop();
    }
}
