//! Tests for scene object ID.

use super::*;

#[test]
fn scene_object_id_equality() {
    assert_eq!(SceneObjectId(1), SceneObjectId(1));
    assert_ne!(SceneObjectId(1), SceneObjectId(2));
}

#[test]
fn scene_object_id_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(SceneObjectId(1), "cube");
    map.insert(SceneObjectId(2), "sphere");
    assert_eq!(map[&SceneObjectId(1)], "cube");
    assert_eq!(map[&SceneObjectId(2)], "sphere");
}

#[test]
fn scene_object_id_copy() {
    let id = SceneObjectId(42);
    let copy = id;
    assert_eq!(id, copy);
}

#[test]
fn scene_object_id_new_unique() {
    let a = SceneObjectId::new();
    let b = SceneObjectId::new();
    let c = SceneObjectId::new();
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn scene_object_id_new_increments() {
    let a = SceneObjectId::new();
    let b = SceneObjectId::new();
    assert_eq!(b.0, a.0 + 1);
}
