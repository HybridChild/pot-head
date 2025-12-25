#![no_std]
#![no_main]

use core::mem::size_of;
use core::panic::PanicInfo;
use pot_head::*;

// Export sizeof values as static symbols that can be extracted with nm/objdump
// These will be readable in the compiled binary

#[unsafe(no_mangle)]
pub static SIZE_CONFIG_U16_F32: usize = size_of::<Config<u16, f32>>();

#[unsafe(no_mangle)]
pub static SIZE_CONFIG_U16_U16: usize = size_of::<Config<u16, u16>>();

#[unsafe(no_mangle)]
pub static SIZE_CONFIG_U8_F32: usize = size_of::<Config<u8, f32>>();

#[unsafe(no_mangle)]
pub static SIZE_STATE_F32: usize = size_of::<State<f32>>();

#[unsafe(no_mangle)]
pub static SIZE_POTHEAD_U16_F32: usize = size_of::<PotHead<u16, f32>>();

#[unsafe(no_mangle)]
pub static SIZE_POTHEAD_U16_U16: usize = size_of::<PotHead<u16, u16>>();

#[unsafe(no_mangle)]
pub static SIZE_POTHEAD_U8_F32: usize = size_of::<PotHead<u8, f32>>();

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Dummy entry point (won't actually be executed)
// References all SIZE_ constants to prevent them from being optimized away
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Volatile read to prevent optimization
    unsafe {
        core::ptr::read_volatile(&SIZE_CONFIG_U16_F32);
        core::ptr::read_volatile(&SIZE_CONFIG_U16_U16);
        core::ptr::read_volatile(&SIZE_CONFIG_U8_F32);
        core::ptr::read_volatile(&SIZE_STATE_F32);
        core::ptr::read_volatile(&SIZE_POTHEAD_U16_F32);
        core::ptr::read_volatile(&SIZE_POTHEAD_U16_U16);
        core::ptr::read_volatile(&SIZE_POTHEAD_U8_F32);
    }
    loop {}
}
