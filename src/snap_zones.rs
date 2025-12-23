//! Snap zone implementations for value snapping and dead zones.
//!
//! Operates on normalized values (0.0-1.0) in the processing pipeline.

/// Snap zone behavior types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapZoneType {
    /// Snap to target value when within threshold
    #[cfg(feature = "snap-zone-snap")]
    Snap,

    /// Dead zone - ignore input changes within threshold
    #[cfg(feature = "snap-zone-dead")]
    Dead,
}

/// Snap zone configuration.
/// Defines a target value and threshold range around it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapZone<T> {
    pub target: T,
    pub threshold: T,
    pub zone_type: SnapZoneType,
}

impl<T> SnapZone<T>
where
    T: Copy + PartialOrd + core::ops::Sub<Output = T> + core::ops::Add<Output = T>,
{
    /// Create a new snap zone
    pub const fn new(target: T, threshold: T, zone_type: SnapZoneType) -> Self {
        Self {
            target,
            threshold,
            zone_type,
        }
    }

    /// Check if value falls within this zone's range (target ± threshold)
    pub fn contains(&self, value: T) -> bool {
        let min = self.target - self.threshold;
        let max = self.target + self.threshold;
        value >= min && value <= max
    }

    /// Apply this zone's behavior to the input value.
    /// Assumes value is within the zone (call contains() first).
    #[allow(unused_variables)]
    pub fn apply(&self, _value: T, last_output: T) -> T {
        match self.zone_type {
            #[cfg(feature = "snap-zone-snap")]
            SnapZoneType::Snap => self.target,

            #[cfg(feature = "snap-zone-dead")]
            SnapZoneType::Dead => last_output,
        }
    }

    /// Check if this zone overlaps with another zone.
    /// Two zones overlap if their ranges (target ± threshold) intersect.
    pub fn overlaps(&self, other: &SnapZone<T>) -> bool {
        let self_min = self.target - self.threshold;
        let self_max = self.target + self.threshold;
        let other_min = other.target - other.threshold;
        let other_max = other.target + other.threshold;

        !(self_max < other_min || other_max < self_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "snap-zone-snap")]
    #[test]
    fn test_snap_zone_contains() {
        let zone = SnapZone::new(0.5, 0.1, SnapZoneType::Snap);

        assert!(zone.contains(0.4)); // min boundary
        assert!(zone.contains(0.5)); // target
        assert!(zone.contains(0.6)); // max boundary
        assert!(!zone.contains(0.39)); // below
        assert!(!zone.contains(0.61)); // above
    }

    #[cfg(feature = "snap-zone-snap")]
    #[test]
    fn test_snap_zone_apply() {
        let zone = SnapZone::new(0.5, 0.1, SnapZoneType::Snap);

        // Snap mode always returns target
        assert_eq!(zone.apply(0.45, 0.0), 0.5);
        assert_eq!(zone.apply(0.55, 0.0), 0.5);
    }

    #[cfg(feature = "snap-zone-dead")]
    #[test]
    fn test_dead_zone_apply() {
        let zone = SnapZone::new(0.5, 0.1, SnapZoneType::Dead);

        // Dead zone returns last output
        assert_eq!(zone.apply(0.45, 0.3), 0.3);
        assert_eq!(zone.apply(0.55, 0.7), 0.7);
    }

    #[cfg(feature = "snap-zone-snap")]
    #[test]
    fn test_snap_zone_overlaps() {
        let zone1 = SnapZone::new(0.0, 0.05, SnapZoneType::Snap); // range: -0.05 to 0.05
        let zone2 = SnapZone::new(0.5, 0.05, SnapZoneType::Snap); // range: 0.45 to 0.55
        let zone3 = SnapZone::new(0.03, 0.03, SnapZoneType::Snap); // range: 0.0 to 0.06

        assert!(!zone1.overlaps(&zone2)); // No overlap
        assert!(!zone2.overlaps(&zone3)); // No overlap
        assert!(zone1.overlaps(&zone3)); // Overlaps
        assert!(zone3.overlaps(&zone1)); // Symmetric
    }

    #[cfg(feature = "snap-zone-snap")]
    #[test]
    fn test_snap_zone_edge_cases() {
        let zone = SnapZone::new(0.0, 0.02, SnapZoneType::Snap);

        // Test negative range (wraps below 0.0)
        assert!(zone.contains(0.0));
        assert!(zone.contains(0.02));
    }
}
