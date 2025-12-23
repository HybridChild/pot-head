//! Grab mode functionality for parameter automation.
//!
//! When physical pot position doesn't match virtual parameter value
//! (e.g., after automation or preset change), grab modes prevent jarring jumps.

/// Grab mode determines how pot position synchronizes with virtual parameter value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrabMode {
    /// Disabled - pot position immediately controls output (may cause jumps).
    None,

    /// Pickup mode - catches when pot crosses virtual value from below.
    /// Industry standard in professional audio equipment.
    Pickup,

    /// PassThrough mode - catches when pot crosses virtual value from either direction.
    /// More intuitive UX - catches from whichever direction you approach.
    PassThrough,
}

impl Default for GrabMode {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grab_mode_default() {
        assert_eq!(GrabMode::default(), GrabMode::None);
    }

    #[test]
    fn test_grab_mode_equality() {
        assert_eq!(GrabMode::None, GrabMode::None);
        assert_eq!(GrabMode::Pickup, GrabMode::Pickup);
        assert_eq!(GrabMode::PassThrough, GrabMode::PassThrough);
        assert_ne!(GrabMode::Pickup, GrabMode::PassThrough);
    }
}
