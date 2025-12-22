use pot_head::curves::ResponseCurve;

#[test]
fn test_linear_curve() {
    let curve = ResponseCurve::Linear;

    assert_eq!(curve.apply(0.0), 0.0);
    assert_eq!(curve.apply(0.25), 0.25);
    assert_eq!(curve.apply(0.5), 0.5);
    assert_eq!(curve.apply(0.75), 0.75);
    assert_eq!(curve.apply(1.0), 1.0);
}

#[cfg(feature = "log-curve")]
#[test]
fn test_logarithmic_curve() {
    let curve = ResponseCurve::Logarithmic;

    // Boundary conditions
    let result_0 = curve.apply(0.0);
    let result_1 = curve.apply(1.0);

    assert!((result_0 - 0.0).abs() < 0.001, "Expected ~0.0, got {}", result_0);
    assert!((result_1 - 1.0).abs() < 0.001, "Expected ~1.0, got {}", result_1);

    // Logarithmic curve should have more resolution at lower values
    // i.e., output at 0.25 should be significantly less than 0.25
    let result_quarter = curve.apply(0.25);
    assert!(result_quarter < 0.15, "Expected <0.15, got {}", result_quarter);

    // Output at 0.5 should be less than 0.5 (shifted down)
    let result_half = curve.apply(0.5);
    assert!(result_half < 0.5, "Expected <0.5, got {}", result_half);

    // Monotonically increasing
    assert!(result_0 < result_quarter);
    assert!(result_quarter < result_half);
    assert!(result_half < result_1);
}

#[cfg(feature = "log-curve")]
#[test]
fn test_logarithmic_curve_clamping() {
    let curve = ResponseCurve::Logarithmic;

    // Values outside range should be clamped
    assert!((curve.apply(-0.1) - 0.0).abs() < 0.001);
    assert!((curve.apply(1.1) - 1.0).abs() < 0.001);
}

#[test]
fn test_response_curve_copy() {
    let curve1 = ResponseCurve::Linear;
    let curve2 = curve1;

    assert_eq!(curve1, curve2);
}
