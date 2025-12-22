use crate::hysteresis::HysteresisState;

pub struct State<T> {
    /// Hysteresis processing state
    pub hysteresis: HysteresisState<T>,
}

impl<T> Default for State<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            hysteresis: HysteresisState::default(),
        }
    }
}
