use cortex_m::asm;

/// RP2350 timer configuration
pub struct Timer;

impl Timer {
    const TIMER_BASE: u32 = 0x400B0000; // Different from RP2040!
    const TIMELR_OFFSET: u32 = 0x28;
    const TIMEHR_OFFSET: u32 = 0x2C;

    /// Initialize timer (already running on RP2350)
    #[inline(never)]
    pub fn init() {
        asm::dmb();
    }

    /// Read current timer value (64-bit at 1 MHz)
    #[inline(never)]
    pub fn read() -> u64 {
        let timer_addr = Self::TIMER_BASE;

        unsafe {
            loop {
                let high1 =
                    core::ptr::read_volatile((timer_addr + Self::TIMEHR_OFFSET) as *const u32);
                let low =
                    core::ptr::read_volatile((timer_addr + Self::TIMELR_OFFSET) as *const u32);
                let high2 =
                    core::ptr::read_volatile((timer_addr + Self::TIMEHR_OFFSET) as *const u32);

                // Check if high word wrapped during read
                if high1 == high2 {
                    asm::dmb();
                    return ((high1 as u64) << 32) | (low as u64);
                }
            }
        }
    }

    /// Convert microseconds to CPU cycles at 150 MHz
    #[inline]
    pub fn micros_to_cycles(micros: u64) -> u64 {
        micros * 150
    }

    /// Get CPU frequency in MHz
    pub const fn cpu_freq_mhz() -> u32 {
        150
    }
}

/// Benchmark measurement helper
pub struct Measurement {
    warmup_iterations: u32,
    measure_iterations: u32,
}

impl Measurement {
    pub const fn new() -> Self {
        Self {
            warmup_iterations: 100,
            measure_iterations: 1000,
        }
    }

    /// Run benchmark with warmup and return average cycles
    #[inline(never)]
    pub fn measure<F>(&self, mut f: F) -> BenchResult
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            f();
            asm::dmb();
        }

        // Measure
        let start = Timer::read();
        asm::dmb();

        for _ in 0..self.measure_iterations {
            f();
            asm::dmb();
        }

        asm::dmb();
        let end = Timer::read();

        let total_micros = end - start;
        let total_cycles = Timer::micros_to_cycles(total_micros);
        let avg_cycles = total_cycles / (self.measure_iterations as u64);
        let avg_micros = total_micros as f32 / self.measure_iterations as f32;

        BenchResult {
            avg_cycles,
            avg_micros,
        }
    }
}

pub struct BenchResult {
    pub avg_cycles: u64,
    pub avg_micros: f32,
}

/// Verify FPU is enabled and active
pub fn validate_fpu() -> bool {
    unsafe {
        // Read CPACR (Coprocessor Access Control Register)
        const CPACR: u32 = 0xE000ED88;
        let cpacr = core::ptr::read_volatile(CPACR as *const u32);

        // Check CP10 and CP11 (FPU) bits [21:20] and [23:22]
        // Should be 0b11 (full access) for both
        let cp10 = (cpacr >> 20) & 0x3;
        let cp11 = (cpacr >> 22) & 0x3;

        asm::dmb();

        cp10 == 0x3 && cp11 == 0x3
    }
}
