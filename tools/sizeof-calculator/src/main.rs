use std::mem::size_of;
use pot_head::*;

fn main() {
    println!("# pot-head sizeof Report\n");
    println!("**Generated:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("**Platform:** {} ({})\n", std::env::consts::OS, std::env::consts::ARCH);

    component_sizes();
    filter_comparison();
    feature_comparison();
    common_configs();
}

fn component_sizes() {
    println!("## Component Sizes\n");
    println!("| Type | Size (bytes) |");
    println!("|------|--------------|");

    // Config sizes for common type combinations
    println!("| Config<u16, f32> | {} |", size_of::<Config<u16, f32>>());
    println!("| Config<u16, u16> | {} |", size_of::<Config<u16, u16>>());
    println!("| Config<u8, f32> | {} |", size_of::<Config<u8, f32>>());

    // State sizes (note: State uses f32 internally regardless of TIn/TOut)
    println!("| State<f32> | {} |", size_of::<State<f32>>());

    // PotHead sizes (Config + State)
    println!("| PotHead<u16, f32> | {} |", size_of::<PotHead<u16, f32>>());
    println!("| PotHead<u16, u16> | {} |", size_of::<PotHead<u16, u16>>());
    println!("| PotHead<u8, f32> | {} |", size_of::<PotHead<u8, f32>>());
    println!();
}

fn filter_comparison() {
    println!("## Filter Impact on RAM\n");
    println!("Understanding filter overhead:\n");
    println!("| Filter | State<f32> Size | RAM Overhead |");
    println!("|--------|-----------------|--------------|");

    let base_size = size_of::<State<f32>>();

    // Note: State size includes space for all possible filters due to struct layout
    // The actual overhead is from the filter's internal state being populated
    println!("| None | {} bytes | baseline |", base_size);
    println!("| EMA | {} bytes | ~8 bytes* |", base_size);

    #[cfg(feature = "moving-average")]
    {
        println!("| MovingAverage(4) | {} bytes | ~32 bytes* |", base_size);
        println!("| MovingAverage(8) | {} bytes | ~48 bytes* |", base_size);
        println!("| MovingAverage(16) | {} bytes | ~80 bytes* |", base_size);
        println!("| MovingAverage(32) | {} bytes | ~144 bytes* |", base_size);
        println!();
        println!("*Note: Moving average overhead scales with window size (4 bytes per sample).");
        println!("The actual State struct size is constant but includes space for the largest buffer.\n");
    }

    #[cfg(not(feature = "moving-average"))]
    {
        println!();
        println!("*Note: EMA filter adds ~8 bytes of initialized state (f32 + bool).");
        println!("Moving average feature is not enabled - add `--features moving-average` to see MovingAverage sizes.\n");
    }
}

fn feature_comparison() {
    println!("## Feature Flag Impact\n");
    println!("Comparing Config + State sizes with different feature combinations:\n");

    println!("| Features | Config<u16,f32> | State<f32> | PotHead<u16,f32> |");
    println!("|----------|-----------------|------------|------------------|");

    // Current build (all features in Cargo.toml)
    let config_size = size_of::<Config<u16, f32>>();
    let state_size = size_of::<State<f32>>();
    let pothead_size = size_of::<PotHead<u16, f32>>();

    #[cfg(all(feature = "std-math", feature = "grab-mode", feature = "moving-average"))]
    println!("| full (current build) | {} | {} | {} |", config_size, state_size, pothead_size);

    #[cfg(all(feature = "std-math", feature = "grab-mode", not(feature = "moving-average")))]
    println!("| default (current build) | {} | {} | {} |", config_size, state_size, pothead_size);

    #[cfg(all(not(feature = "std-math"), not(feature = "grab-mode"), not(feature = "moving-average")))]
    println!("| minimal (current build) | {} | {} | {} |", config_size, state_size, pothead_size);

    println!();
    println!("**Feature breakdown:**");
    println!("- `minimal`: No features (--no-default-features)");
    println!("- `default`: std-math + grab-mode");
    println!("- `full`: std-math + grab-mode + moving-average");
    println!();
    println!("**Note:** This tool was built with all features. To see minimal/default sizes,");
    println!("rebuild with different feature flags and run again.\n");
}

fn common_configs() {
    println!("## Common Embedded Configurations\n");
    println!("Real-world usage scenarios:\n");
    println!("| Use Case | Type Combo | Filter | Features Needed | Est. Total RAM |");
    println!("|----------|------------|--------|-----------------|----------------|");

    let base_pothead = size_of::<PotHead<u16, f32>>();

    println!("| Simple volume knob | u16→f32 | EMA | minimal | ~{} bytes |", base_pothead);
    println!("| Audio taper control | u16→f32 | EMA | std-math | ~{} bytes |", base_pothead);
    println!("| Integer scaler | u16→u16 | None | minimal | ~{} bytes |", size_of::<PotHead<u16, u16>>());

    #[cfg(feature = "grab-mode")]
    {
        println!("| Synth parameter (w/ pickup) | u16→f32 | EMA | default | ~{} bytes |", base_pothead);
    }

    #[cfg(feature = "moving-average")]
    {
        println!("| High-precision w/ MA(16) | u16→f32 | MA(16) | full | ~{} bytes |", base_pothead);
    }

    println!();
    println!("## Summary\n");

    let min_config = size_of::<PotHead<u16, u16>>();
    let typical_config = size_of::<PotHead<u16, f32>>();

    println!("- **Minimal config**: ~{} bytes (u16→u16, no filter, no features)", min_config);
    println!("- **Typical config**: ~{} bytes (u16→f32, EMA filter)", typical_config);
    println!("- **Maximum config**: ~{} bytes (all features, largest filter)", typical_config);
}
