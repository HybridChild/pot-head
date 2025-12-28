#![no_std]
#![no_main]

use panic_halt as _;
use rp235x_hal::entry;
use rtt_target::{rprintln, rtt_init_print};

mod bench;
mod scenarios;

use bench::{Measurement, Timer, validate_fpu};
use pot_head::*;
use scenarios::*;

// RP2350 boot block - required for the chip to boot
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: rp235x_hal::block::ImageDef = rp235x_hal::block::ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();

    rprintln!("");
    rprintln!("pot-head Hardware Benchmark");
    rprintln!("===========================");
    rprintln!("");
    rprintln!("Platform: RP2350 (Cortex-M33F)");
    rprintln!(
        "CPU: {} MHz, FPU: Single-precision (hard-float)",
        Timer::cpu_freq_mhz()
    );
    rprintln!("Target: thumbv8m.main-none-eabihf");
    rprintln!("FPU Enabled: {}", validate_fpu());
    rprintln!("");
    rprintln!("PotHead::update() Performance");
    rprintln!("-----------------------------");
    rprintln!("");
    rprintln!("Scenario              Cycles      µs");
    rprintln!("==================  ========  ======");

    Timer::init();
    let measurement = Measurement::new();

    // Run benchmarks
    run_benchmark(&measurement, "Baseline", || bench_baseline());

    run_benchmark(&measurement, "With EMA", || bench_with_ema());

    #[cfg(feature = "std-math")]
    run_benchmark(&measurement, "With Log Curve", || bench_with_log_curve());

    run_benchmark(&measurement, "Full Featured", || bench_full_featured());

    #[cfg(feature = "moving-average")]
    run_benchmark(&measurement, "MA Window=4", || bench_ma_window_4());

    #[cfg(feature = "moving-average")]
    run_benchmark(&measurement, "MA Window=16", || bench_ma_window_16());

    run_benchmark(&measurement, "u16→u16", || bench_u16_to_u16());

    rprintln!("");
    rprintln!("Benchmark complete.");

    loop {
        cortex_m::asm::wfi();
    }
}

#[inline(never)]
fn run_benchmark<F>(measurement: &Measurement, name: &str, f: F)
where
    F: FnMut(),
{
    let result = measurement.measure(f);

    rprintln!(
        "{:<18}  {:>8}  {:>6.2}",
        name,
        result.avg_cycles,
        result.avg_micros
    );
}

// Benchmark functions with #[inline(never)] to prevent optimization

#[inline(never)]
fn bench_baseline() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&BASELINE).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[inline(never)]
fn bench_with_ema() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&WITH_EMA).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[cfg(feature = "std-math")]
#[inline(never)]
fn bench_with_log_curve() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&WITH_LOG_CURVE).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[inline(never)]
fn bench_full_featured() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&FULL_FEATURED).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[cfg(feature = "moving-average")]
#[inline(never)]
fn bench_ma_window_4() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&MA_WINDOW_4).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[cfg(feature = "moving-average")]
#[inline(never)]
fn bench_ma_window_16() {
    static mut POT: Option<PotHead<u16, f32>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&MA_WINDOW_16).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_F32, output);
    }
}

#[inline(never)]
fn bench_u16_to_u16() {
    static mut POT: Option<PotHead<u16, u16>> = None;

    unsafe {
        let pot_ptr = core::ptr::addr_of_mut!(POT);
        if (*pot_ptr).is_none() {
            *pot_ptr = Some(PotHead::new(&U16_TO_U16).unwrap());
        }

        let input = core::ptr::read_volatile(&2048u16);
        let output = (*pot_ptr).as_mut().unwrap().update(input);
        core::ptr::write_volatile(&raw mut OUTPUT_U16, output);
    }
}

// Black box outputs to prevent dead code elimination
static mut OUTPUT_F32: f32 = 0.0;
static mut OUTPUT_U16: u16 = 0;
