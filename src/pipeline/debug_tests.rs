use super::*;

#[test]
fn uniform_size_is_16() {
    assert_eq!(UNIFORM_SIZE, 16);
}

#[test]
fn mode_count_is_6() {
    assert_eq!(MODE_COUNT, 6);
}

#[test]
fn uniforms_length_matches_size() {
    let bytes = uniforms(0);
    assert_eq!(bytes.len(), UNIFORM_SIZE);
}

#[test]
fn uniforms_mode_encoded_as_first_f32() {
    for mode in 0..MODE_COUNT {
        let bytes = uniforms(mode);
        let first = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert!((first - mode as f32).abs() < 1e-6, "mode {mode}");
    }
}

#[test]
fn uniforms_padding_is_zero() {
    let bytes = uniforms(3);
    // Bytes 4..16 should be zero (three padding f32s)
    for (i, &byte) in bytes.iter().enumerate().skip(4) {
        assert_eq!(byte, 0, "byte {i} should be zero padding");
    }
}

#[test]
fn fragment_wgsl_non_empty() {
    assert!(!FRAGMENT_WGSL.is_empty());
}

#[test]
fn fragment_wgsl_has_entry_point() {
    assert!(FRAGMENT_WGSL.contains("fs_main"));
}
