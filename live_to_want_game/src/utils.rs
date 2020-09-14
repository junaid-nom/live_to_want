use std::sync::{atomic::AtomicU64};

#[derive(Debug)]
#[derive(Default)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}

pub static COUNTER: AtomicU64 = AtomicU64::new(1); // TODO: Upgrade to a 128 bit one when it comes out of nightly build
pub fn get_id() -> u64 { COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }
pub type UID = u64;

