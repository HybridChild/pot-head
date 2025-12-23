use num_traits::AsPrimitive;

use crate::config::{Config, ConfigError};
use crate::filters::NoiseFilter;
use crate::state::State;

#[cfg(feature = "grab-mode")]
use crate::grab_mode::GrabMode;

#[cfg(feature = "filter-ema")]
use crate::filters::EmaFilter;
#[cfg(feature = "filter-moving-avg")]
use crate::filters::MovingAvgFilter;

pub struct PotHead<TIn, TOut = TIn> {
    config: Config<TIn, TOut>,
    state: State<f32>,
}

impl<TIn, TOut> PotHead<TIn, TOut>
where
    TIn: Copy + PartialOrd + AsPrimitive<f32>,
    TOut: Copy + PartialOrd + AsPrimitive<f32>,
    f32: AsPrimitive<TOut>,
{
    pub fn new(config: Config<TIn, TOut>) -> Result<Self, ConfigError> {
        config.validate()?;

        let mut state = State::default();

        // Initialize filter state based on configuration
        #[cfg(feature = "filter-ema")]
        if matches!(config.filter, NoiseFilter::ExponentialMovingAverage { .. }) {
            state.ema_filter = Some(EmaFilter::new());
        }

        #[cfg(feature = "filter-moving-avg")]
        if let NoiseFilter::MovingAverage { window_size } = config.filter {
            state.ma_filter = Some(MovingAvgFilter::new(window_size));
        }

        Ok(Self { config, state })
    }

    pub fn config(&self) -> &Config<TIn, TOut> {
        &self.config
    }

    pub fn update(&mut self, input: TIn) -> TOut {
        // Normalize input to 0.0..1.0
        let normalized = self.normalize_input(input);

        // Apply noise filter
        let filtered = self.apply_filter(normalized);

        // Apply response curve
        let curved = self.config.curve.apply(filtered);

        // Apply hysteresis
        let hysteresis_applied = self
            .config
            .hysteresis
            .apply(curved, &mut self.state.hysteresis);

        // Capture physical position BEFORE snap zones and grab mode
        #[cfg(feature = "grab-mode")]
        {
            self.state.physical_position = hysteresis_applied;
        }

        // Apply snap zones
        let snapped = self.apply_snap_zones(hysteresis_applied);

        // Apply grab mode logic
        #[cfg(feature = "grab-mode")]
        let output = self.apply_grab_mode(snapped);

        #[cfg(not(feature = "grab-mode"))]
        let output = snapped;

        // Update last output for dead zones
        self.state.last_output = output;

        // Denormalize to output range
        self.denormalize_output(output)
    }

    fn apply_filter(&mut self, value: f32) -> f32 {
        match &self.config.filter {
            NoiseFilter::None => value,

            #[cfg(feature = "filter-ema")]
            NoiseFilter::ExponentialMovingAverage { alpha } => {
                if let Some(ref mut filter) = self.state.ema_filter {
                    filter.apply(value, *alpha)
                } else {
                    value
                }
            }

            #[cfg(feature = "filter-moving-avg")]
            NoiseFilter::MovingAverage { .. } => {
                if let Some(ref mut filter) = self.state.ma_filter {
                    filter.apply(value)
                } else {
                    value
                }
            }
        }
    }

    fn apply_snap_zones(&self, value: f32) -> f32 {
        // Process zones in order - first match wins
        for zone in self.config.snap_zones {
            if zone.contains(value) {
                return zone.apply(value, self.state.last_output);
            }
        }
        value // No zone matched
    }

    fn normalize_input(&self, input: TIn) -> f32 {
        let input_f = input.as_();
        let min_f = self.config.input_min.as_();
        let max_f = self.config.input_max.as_();

        // Clamp input to valid range
        let clamped = if input_f < min_f {
            min_f
        } else if input_f > max_f {
            max_f
        } else {
            input_f
        };

        // Normalize to 0.0..1.0
        // Safe division: validation ensures max_f > min_f
        (clamped - min_f) / (max_f - min_f)
    }

    fn denormalize_output(&self, normalized: f32) -> TOut {
        let min_f = self.config.output_min.as_();
        let max_f = self.config.output_max.as_();

        let output_f = min_f + normalized * (max_f - min_f);
        output_f.as_()
    }

    #[cfg(feature = "grab-mode")]
    fn apply_grab_mode(&mut self, value: f32) -> f32 {
        match self.config.grab_mode {
            GrabMode::None => {
                // Direct control, no grab logic
                self.state.grabbed = true; // Always consider grabbed
                self.state.virtual_value = value;
                value
            }

            GrabMode::Pickup => {
                if !self.state.grabbed {
                    // Check if pot crosses virtual value from below
                    if value >= self.state.virtual_value {
                        self.state.grabbed = true;
                    } else {
                        // Hold virtual value until grabbed
                        return self.state.virtual_value;
                    }
                }
                // Pot is grabbed - update virtual value
                self.state.virtual_value = value;
                value
            }

            GrabMode::PassThrough => {
                if !self.state.grabbed {
                    // First read after set_virtual_value - just initialize position
                    if !self.state.passthrough_initialized {
                        self.state.last_physical = value;
                        self.state.passthrough_initialized = true;
                        return self.state.virtual_value;
                    }

                    // Check if pot crosses virtual value from either direction
                    let crossing_from_below = value >= self.state.virtual_value
                        && self.state.last_physical < self.state.virtual_value;

                    let crossing_from_above = value <= self.state.virtual_value
                        && self.state.last_physical > self.state.virtual_value;

                    if crossing_from_below || crossing_from_above {
                        self.state.grabbed = true;
                        self.state.last_physical = value;
                        self.state.virtual_value = value;
                        return value;
                    }

                    // Not grabbed yet - update last physical and hold virtual value
                    self.state.last_physical = value;
                    return self.state.virtual_value;
                }
                // Pot is grabbed - update both physical and virtual
                self.state.last_physical = value;
                self.state.virtual_value = value;
                value
            }
        }
    }

    /// Returns the current physical input position in normalized 0.0..1.0 range.
    /// Useful for UI display when grab mode is active.
    ///
    /// This always reflects where the pot physically is (after normalize→filter→curve→hysteresis),
    /// but BEFORE virtual modifications like snap zones and grab mode logic.
    #[cfg(feature = "grab-mode")]
    pub fn physical_position(&self) -> f32 {
        self.state.physical_position
    }

    /// Returns the current output value in normalized 0.0..1.0 range without updating state.
    /// Useful for reading the locked virtual value in grab mode.
    #[cfg(feature = "grab-mode")]
    pub fn current_output(&self) -> f32 {
        self.state.virtual_value
    }

    /// Returns true if grab mode is active but not yet grabbed.
    /// When true, `physical_position() != current_output()`
    #[cfg(feature = "grab-mode")]
    pub fn is_waiting_for_grab(&self) -> bool {
        matches!(
            self.config.grab_mode,
            GrabMode::Pickup | GrabMode::PassThrough
        ) && !self.state.grabbed
    }

    /// Set the virtual parameter value (e.g., after preset change or automation).
    /// This unlocks grab mode, requiring the pot to be grabbed again.
    #[cfg(feature = "grab-mode")]
    pub fn set_virtual_value(&mut self, value: f32) {
        self.state.virtual_value = value;
        self.state.grabbed = false;
        self.state.passthrough_initialized = false; // Reset for PassThrough mode
    }

    /// Release grab and set virtual value to current physical position.
    /// Useful when switching which parameter a physical pot controls.
    ///
    /// After calling this, the pot will be ungrabbed and the virtual value
    /// will be set to the current physical position. The pot must be moved
    /// to re-grab (in Pickup/PassThrough modes).
    #[cfg(feature = "grab-mode")]
    pub fn release(&mut self) {
        self.state.virtual_value = self.state.physical_position;
        self.state.grabbed = false;
        self.state.passthrough_initialized = false;
    }
}
