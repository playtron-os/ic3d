use super::*;

#[test]
fn parse_rect() {
    let ring = parse_path("M0 0 L10 0 L10 10 L0 10 Z", 8);
    assert_eq!(ring.len(), 4);
    assert_eq!(ring[0], [0.0, 0.0]);
    assert_eq!(ring[1], [10.0, 0.0]);
    assert_eq!(ring[2], [10.0, 10.0]);
    assert_eq!(ring[3], [0.0, 10.0]);
}

#[test]
fn parse_h_and_v() {
    let ring = parse_path("M0 0 H10 V10 H0 V0 Z", 8);
    assert_eq!(ring.len(), 5); // M + H + V + H + V
    assert_eq!(ring[0], [0.0, 0.0]);
    assert_eq!(ring[1], [10.0, 0.0]);
    assert_eq!(ring[2], [10.0, 10.0]);
    assert_eq!(ring[3], [0.0, 10.0]);
    assert_eq!(ring[4], [0.0, 0.0]);
}

#[test]
fn parse_cubic_adds_segments() {
    // Simple curve: M0 0 C5 10 10 10 15 0
    let ring = parse_path("M0 0 C5 10 10 10 15 0", 8);
    // 1 from M + 8 from C
    assert_eq!(ring.len(), 9);
    // First point is the M
    assert_eq!(ring[0], [0.0, 0.0]);
    // Last point should be the C endpoint
    assert!((ring[8][0] - 15.0).abs() < 1e-5);
    assert!(ring[8][1].abs() < 1e-5);
}

#[test]
fn parse_negative_numbers() {
    let ring = parse_path("M-5 -10 L5 10 Z", 8);
    assert_eq!(ring.len(), 2);
    assert_eq!(ring[0], [-5.0, -10.0]);
    assert_eq!(ring[1], [5.0, 10.0]);
}

#[test]
fn parse_commas_and_whitespace() {
    let ring = parse_path("M0,0 L10,0 L10,10 L0,10Z", 8);
    assert_eq!(ring.len(), 4);
}

#[test]
fn parse_m_implicit_lineto() {
    // After M, subsequent number pairs are implicit L commands
    let ring = parse_path("M0 0 10 0 10 10 0 10 Z", 8);
    assert_eq!(ring.len(), 4);
    assert_eq!(ring[3], [0.0, 10.0]);
}

#[test]
fn parse_empty_path() {
    let ring = parse_path("", 8);
    assert!(ring.is_empty());
}

#[test]
fn parse_curve_segments_controls_resolution() {
    let path = "M0 0 C5 10 10 10 15 0";
    let low = parse_path(path, 4);
    let high = parse_path(path, 16);
    assert_eq!(low.len(), 5); // 1 M + 4 segments
    assert_eq!(high.len(), 17); // 1 M + 16 segments
}

#[test]
fn parse_real_svg_letter() {
    // Simplified excerpt from a real letter (L shape: rect)
    let ring = parse_path("M51.7772 20H60.4464V39.206H51.7772V20Z", 8);
    assert_eq!(ring.len(), 5); // M + H + V + H + V
    assert!((ring[0][0] - 51.7772).abs() < 1e-3);
    assert!((ring[0][1] - 20.0).abs() < 1e-3);
}
