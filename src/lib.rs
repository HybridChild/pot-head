#![no_std]

mod config;
mod state;
mod pothead;
pub mod hysteresis;

pub use config::{Config, ConfigError};
pub use state::State;
pub use pothead::PotHead;
pub use hysteresis::{HysteresisMode, HysteresisState};

#[cfg(feature = "hysteresis-schmitt")]
pub use hysteresis::SchmittState;
