#![no_std]
#![no_main]

use core::panic::PanicInfo;
use pot_head::*;

// Static ROM configuration (v0.1 pattern)
// Exercises all major code paths to prevent dead code elimination
static CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095, // 12-bit ADC

    output_min: 0.0,
    output_max: 1.0,

    // Feature-gated curve selection
    #[cfg(feature = "std-math")]
    curve: ResponseCurve::Logarithmic,
    #[cfg(not(feature = "std-math"))]
    curve: ResponseCurve::Linear,

    // EMA filter (always available)
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },

    // Change threshold hysteresis
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.01 },

    // Snap zone at center
    snap_zones: &[SnapZone::new(0.5, 0.02, SnapZoneType::Snap)],

    // Feature-gated grab mode
    #[cfg(feature = "grab-mode")]
    grab_mode: GrabMode::Pickup,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Entry point that actually uses PotHead to measure code size
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Create PotHead instance (error path also exercised)
    let mut pot = match PotHead::new(&CONFIG) {
        Ok(p) => p,
        Err(_) => loop {}, // Error case (shouldn't happen with valid CONFIG)
    };

    // Exercise update path with volatile operations
    // to prevent LLVM from optimizing away the entire logic
    let input = unsafe { core::ptr::read_volatile(&2048u16) };
    let output = pot.process(input);

    // Store result to prevent optimization
    unsafe {
        core::ptr::write_volatile(&mut OUTPUT as *mut f32, output);
    }

    loop {}
}

// Static mutable for output storage (prevents DCE)
static mut OUTPUT: f32 = 0.0;
