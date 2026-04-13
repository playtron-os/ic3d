use super::*;

#[test]
fn hash_f32_in_range() {
    for i in 0..20 {
        let h = hash_f32(i as f32, (i * 7) as f32, 42.0);
        assert!((0.0..1.0).contains(&h), "hash={h} out of [0, 1)");
    }
}

#[test]
fn hash_f32_deterministic() {
    let a = hash_f32(1.0, 2.0, 3.0);
    let b = hash_f32(1.0, 2.0, 3.0);
    assert_eq!(a, b);
}

#[test]
fn hash_f32_signed_in_range() {
    for i in 0..20 {
        let h = hash_f32_signed(i as f32, (i * 3) as f32, 0.0);
        assert!((-1.0..1.0).contains(&h), "hash_signed={h} out of (-1, 1)");
    }
}

#[test]
fn hash_f32_range_in_bounds() {
    for i in 0..20 {
        let h = hash_f32_range(i as f32, (i * 5) as f32, 1.0, 10.0, 20.0);
        assert!((10.0..20.0).contains(&h), "hash_range={h} out of [10, 20)");
    }
}
