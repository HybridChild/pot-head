#![no_std]

mod config;
mod state;
mod pothead;
pub mod hysteresis;
pub mod curves;
pub mod filters;
pub mod snap_zones;

pub use config::{Config, ConfigError};
pub use state::State;
pub use pothead::PotHead;
pub use hysteresis::{HysteresisMode, HysteresisState};
pub use curves::ResponseCurve;
pub use filters::NoiseFilter;
pub use snap_zones::{SnapZone, SnapZoneType};

#[cfg(feature = "hysteresis-schmitt")]
pub use hysteresis::SchmittState;
