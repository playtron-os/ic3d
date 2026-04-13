use super::*;

#[test]
fn distance_2d_zero() {
    assert_eq!(distance_2d(0.0, 0.0, 0.0, 0.0), 0.0);
}

#[test]
fn distance_2d_unit() {
    assert!((distance_2d(0.0, 0.0, 3.0, 4.0) - 5.0).abs() < f32::EPSILON);
}

#[test]
fn distance_2d_symmetric() {
    let d1 = distance_2d(1.0, 2.0, 4.0, 6.0);
    let d2 = distance_2d(4.0, 6.0, 1.0, 2.0);
    assert!((d1 - d2).abs() < f32::EPSILON);
}

#[test]
fn flatten_cubic_segment_count() {
    let mut pts = Vec::new();
    flatten_cubic([0.0, 0.0], [0.5, 1.0], [1.0, 1.0], [1.5, 0.0], 8, &mut pts);
    assert_eq!(pts.len(), 8);
}

#[test]
fn flatten_cubic_endpoints() {
    let mut pts = Vec::new();
    flatten_cubic([0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], 16, &mut pts);
    // Last point should be the end control point
    let last = pts.last().unwrap();
    assert!((last[0] - 1.0).abs() < 1e-5);
    assert!(last[1].abs() < 1e-5);
}

#[test]
fn flatten_cubic_straight_line() {
    // When all control points are collinear, output should be a straight line
    let mut pts = Vec::new();
    flatten_cubic([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], 4, &mut pts);
    for pt in &pts {
        assert!(pt[1].abs() < 1e-6, "y should be 0 for collinear points");
    }
    // x should be monotonically increasing
    for i in 1..pts.len() {
        assert!(pts[i][0] > pts[i - 1][0]);
    }
}

#[test]
fn flatten_cubic_appends() {
    let mut pts = vec![[0.0, 0.0]]; // existing start point
    flatten_cubic([0.0, 0.0], [0.5, 1.0], [1.0, 1.0], [1.5, 0.0], 4, &mut pts);
    assert_eq!(pts.len(), 5); // 1 existing + 4 new
}
