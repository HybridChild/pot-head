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
}

impl<T> Default for State<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            hysteresis: HysteresisState::default(),
            #[cfg(feature = "filter-ema")]
            ema_filter: None,
            #[cfg(feature = "filter-moving-avg")]
            ma_filter: None,
        }
    }
}
