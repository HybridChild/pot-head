#![no_std]

mod config;
pub mod curves;
pub mod filters;
pub mod hysteresis;
mod pothead;
pub mod snap_zones;
mod state;

#[cfg(feature = "grab-mode")]
pub mod grab_mode;

pub use config::{Config, ConfigError};
pub use curves::ResponseCurve;
pub use filters::NoiseFilter;
pub use hysteresis::{HysteresisMode, HysteresisState, SchmittState};
pub use pothead::PotHead;
pub use snap_zones::{SnapZone, SnapZoneType};
pub use state::State;

#[cfg(feature = "grab-mode")]
pub use grab_mode::GrabMode;
