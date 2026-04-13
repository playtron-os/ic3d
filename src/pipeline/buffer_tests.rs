use super::*;

#[test]
fn pool_new_is_empty() {
    let pool = BufferPool::new();
    assert_eq!(pool.frame(), 0);
    assert_eq!(pool.pending_count(), 0);
    assert_eq!(pool.available_count(), 0);
}

#[test]
fn pool_with_latency() {
    let pool = BufferPool::with_latency(5);
    assert_eq!(pool.frame(), 0);
}

#[test]
fn pool_default_equals_new() {
    let a = BufferPool::new();
    let b = BufferPool::default();
    assert_eq!(a.frame(), b.frame());
    assert_eq!(a.pending_count(), b.pending_count());
    assert_eq!(a.available_count(), b.available_count());
}

#[test]
fn pool_advance_frame_increments() {
    let mut pool = BufferPool::new();
    pool.advance_frame();
    assert_eq!(pool.frame(), 1);
    pool.advance_frame();
    assert_eq!(pool.frame(), 2);
}

#[test]
fn pool_advance_no_crash_when_empty() {
    let mut pool = BufferPool::new();
    for _ in 0..100 {
        pool.advance_frame();
    }
    assert_eq!(pool.frame(), 100);
    assert_eq!(pool.pending_count(), 0);
    assert_eq!(pool.available_count(), 0);
}
