use std::sync::{atomic::AtomicU64};

#[derive(Debug)]
#[derive(Default)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
// TODO: Convert to usize. need to do checked_add/checked_sub everywhere
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}
impl Vector2 {
    pub fn new(x: i32, y: i32) -> Self {
        Vector2{ x, y }
    }
    pub fn get_neighbors(&self, diagonals: bool) -> Vec<Vector2> {
        let up = Vector2 {x: self.x, y: self.y+1};
        let down = Vector2 {x: self.x, y: self.y-1};
        let right = Vector2 {x: self.x+1, y: self.y};
        let left = Vector2 {x: self.x-1, y: self.y};
        if !diagonals {
            return vec![right, up, down, left];
        }
        let upright = Vector2 {x: self.x+1, y: self.y+1};
        let downright = Vector2 {x: self.x+1, y: self.y-1};
        let downleft = Vector2 {x: self.x-1, y: self.y-1};
        let upleft = Vector2 {x: self.x-1, y: self.y+1};
        // TODO: return shuffled?
        return vec![right, up, down, left, upright, downright, downleft, upleft];
    }
}

pub static COUNTER: AtomicU64 = AtomicU64::new(1); // TODO: Upgrade to a 128 bit one when it comes out of nightly build
pub fn get_id() -> u64 { COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }
pub type UID = u64;

