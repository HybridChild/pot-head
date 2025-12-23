use crate::hysteresis::HysteresisState;

#[cfg(feature = "filter-ema")]
use crate::filters::EmaFilter;
#[cfg(feature = "filter-moving-avg")]
use crate::filters::MovingAvgFilter;

pub struct State<T> {
    /// Hysteresis processing state
    pub hysteresis: HysteresisState<T>,

    /// EMA filter state
    #[cfg(feature = "filter-ema")]
    pub ema_filter: Option<EmaFilter>,

    /// Moving average filter state
    #[cfg(feature = "filter-moving-avg")]
    pub ma_filter: Option<MovingAvgFilter>,

    /// Last output value (for dead zones)
    pub last_output: T,

    /// Grab mode: whether pot has been grabbed
    #[cfg(feature = "grab-mode")]
    pub grabbed: bool,

    /// Grab mode: virtual parameter value (locked when not grabbed)
    #[cfg(feature = "grab-mode")]
    pub virtual_value: T,

    /// Grab mode: physical position after processing (before snap zones)
    #[cfg(feature = "grab-mode")]
    pub physical_position: T,

    /// Grab mode: last physical position (for PassThrough crossing detection)
    #[cfg(feature = "grab-mode")]
    pub last_physical: T,

    /// Grab mode: whether we've initialized last_physical (for PassThrough first read)
    #[cfg(feature = "grab-mode")]
    pub passthrough_initialized: bool,
}

impl<T> Default for State<T>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self {
            hysteresis: HysteresisState::default(),
            #[cfg(feature = "filter-ema")]
            ema_filter: None,
            #[cfg(feature = "filter-moving-avg")]
            ma_filter: None,
            last_output: T::default(),
            #[cfg(feature = "grab-mode")]
            grabbed: false,
            #[cfg(feature = "grab-mode")]
            virtual_value: T::default(),
            #[cfg(feature = "grab-mode")]
            physical_position: T::default(),
            #[cfg(feature = "grab-mode")]
            last_physical: T::default(),
            #[cfg(feature = "grab-mode")]
            passthrough_initialized: false,
        }
    }
}
