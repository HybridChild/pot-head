use crate::color_scheme::ColorScheme;
use crate::pot_adapter::PotAdapter;
use crate::renderable_pot::RenderablePot;
use crossterm::style::Color;
use num_traits::AsPrimitive;
use pot_head::{
    Config, GrabMode, HysteresisMode, NoiseFilter, PotHead, ResponseCurve, SnapZone, SnapZoneType,
};
use std::fmt::Display;
use std::io::Result;

// Default color scheme for all pots
const DEFAULT_COLOR_SCHEME: ColorScheme = ColorScheme {
    bar_color: Color::Rgb { r: 0, g: 255, b: 0 },
    processed_indicator_color: Color::Rgb {
        r: 0,
        g: 200,
        b: 255,
    },
    physical_indicator_color: Color::Rgb {
        r: 255,
        g: 165,
        b: 0,
    },
    threshold_color: Color::Rgb {
        r: 150,
        g: 150,
        b: 150,
    },
    snap_zone_color: Color::Rgb {
        r: 100,
        g: 200,
        b: 255,
    }, // Light blue for snap zones
    dead_zone_color: Color::Rgb {
        r: 100,
        g: 100,
        b: 100,
    }, // Gray for dead zones
};

/// Specification for creating a pot with all its display properties
pub struct PotSpec<TIn: 'static, TOut: 'static> {
    pub label: &'static str,
    pub config: &'static Config<TIn, TOut>,
    pub color_scheme: ColorScheme,
    pub precision: usize,
}

impl<TIn, TOut> PotSpec<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32> + 'static,
    TOut: Copy + PartialOrd + AsPrimitive<f32> + Display + 'static,
    f32: AsPrimitive<TIn> + AsPrimitive<TOut>,
{
    pub fn build(&self) -> Result<Box<dyn RenderablePot>> {
        let pot = PotHead::new(self.config).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} config error: {:?}", self.label, e),
            )
        })?;

        Ok(Box::new(PotAdapter::new(
            pot,
            self.label,
            self.color_scheme,
            self.precision,
            self.config.input_min,
            self.config.input_max,
        )))
    }
}

// Empty snap zones for pots that don't use them
static EMPTY_SNAP_ZONES: [SnapZone<f32>; 0] = [];

// Static configurations
static RAW_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static REVERSED_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 100.0,
    output_max: -100.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static SCHMITT_POT_CONFIG: Config<u16, i32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0,
    output_max: 127,
    hysteresis: HysteresisMode::SchmittTrigger {
        rising: 0.6,
        falling: 0.4,
    },
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static LOG_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: -60.0,
    output_max: 0.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Logarithmic,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

static FILTERED_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::ChangeThreshold { threshold: 0.05 },
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::ExponentialMovingAverage { alpha: 0.3 },
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::None,
};

// Snap zones configuration for SNAP_POT
static SNAP_POT_ZONES: [SnapZone<f32>; 3] = [
    SnapZone::new(0.0, 0.1, SnapZoneType::Snap), // Snap to 0% (±10%)
    SnapZone::new(0.5, 0.1, SnapZoneType::Dead), // Dead zone at 50% (±10%)
    SnapZone::new(1.0, 0.1, SnapZoneType::Snap), // Snap to 100% (±10%)
];

static SNAP_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &SNAP_POT_ZONES,
    grab_mode: GrabMode::None,
};

static _PICKUP_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::Pickup,
};

static PASSTHROUGH_POT_CONFIG: Config<u16, f32> = Config {
    input_min: 0,
    input_max: 4095,
    output_min: 0.0,
    output_max: 1.0,
    hysteresis: HysteresisMode::none(),
    curve: ResponseCurve::Linear,
    filter: NoiseFilter::None,
    snap_zones: &EMPTY_SNAP_ZONES,
    grab_mode: GrabMode::PassThrough,
};

// Pre-defined pot specifications
pub const RAW_POT: PotSpec<u16, f32> = PotSpec {
    label: "Raw Pot",
    config: &RAW_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};

pub const REVERSED_POT: PotSpec<u16, f32> = PotSpec {
    label: "Reversed Pot",
    config: &REVERSED_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 2,
};

pub const SCHMITT_POT: PotSpec<u16, i32> = PotSpec {
    label: "Schmitt Pot",
    config: &SCHMITT_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 0,
};

pub const LOG_POT: PotSpec<u16, f32> = PotSpec {
    label: "Log Pot (Audio Taper)",
    config: &LOG_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};

pub const FILTERED_POT: PotSpec<u16, f32> = PotSpec {
    label: "Filtered Pot (EMA)",
    config: &FILTERED_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};

pub const SNAP_POT: PotSpec<u16, f32> = PotSpec {
    label: "Snap Zones Pot",
    config: &SNAP_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};

pub const _PICKUP_POT: PotSpec<u16, f32> = PotSpec {
    label: "Pickup Mode Pot",
    config: &_PICKUP_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};

pub const PASSTHROUGH_POT: PotSpec<u16, f32> = PotSpec {
    label: "PassThrough Mode Pot",
    config: &PASSTHROUGH_POT_CONFIG,
    color_scheme: DEFAULT_COLOR_SCHEME,
    precision: 3,
};
