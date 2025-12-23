#![no_std]

mod config;
mod state;
mod pothead;
pub mod hysteresis;
pub mod curves;
pub mod filters;
pub mod snap_zones;

#[cfg(feature = "grab-mode")]
pub mod grab_mode;

pub use config::{Config, ConfigError};
pub use state::State;
pub use pothead::PotHead;
pub use hysteresis::{HysteresisMode, HysteresisState};
pub use curves::ResponseCurve;
pub use filters::NoiseFilter;
pub use snap_zones::{SnapZone, SnapZoneType};

#[cfg(feature = "grab-mode")]
pub use grab_mode::GrabMode;

#[cfg(feature = "hysteresis-schmitt")]
pub use hysteresis::SchmittState;
