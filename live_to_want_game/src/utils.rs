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
    pub fn add(v1: &Vector2, v2: &Vector2) -> Vector2 {
        Vector2::new(v1.x + v2.x, v1.y + v2.y)
    }
}

#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Neighbor {
    Left(Vu2),
    Right(Vu2),
    Down(Vu2),
    Up(Vu2),
}
impl Neighbor {
    pub fn get(&self) -> Vu2{
        match &self {
            Neighbor::Left(v) => {*v}
            Neighbor::Right(v) => {*v}
            Neighbor::Down(v) => {*v}
            Neighbor::Up(v) => {*v}
        }
    }
    pub fn opposite(&self) -> Neighbor {
        match &self {
            Neighbor::Left(v) => {Neighbor::Right(*v)}
            Neighbor::Right(v) => {Neighbor::Left(*v)}
            Neighbor::Down(v) => {Neighbor::Up(*v)}
            Neighbor::Up(v) => {Neighbor::Down(*v)}
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
// TODO: Convert to usize. need to do checked_add/checked_sub everywhere
pub struct Vu2 {
    pub x: usize,
    pub y: usize,
}
impl Vu2 {
    pub fn new(x: usize, y: usize) -> Self {
        Vu2{ x, y }
    }
    pub fn add(v1: &Vu2, v2: &Vu2) -> Vu2 {
        Vu2::new(v1.x + v2.x, v1.y + v2.y)
    }
    pub fn get_valid_neighbors(&self, xlen: usize, ylen: usize) -> Vec<Neighbor> {
        let mut ret = Vec::new();
        if self.y+1 < ylen {
            ret.push(Neighbor::Up(Vu2::new(self.x, self.y+1)));
        }
        if self.x+1 < xlen {
            ret.push(Neighbor::Right(Vu2::new(self.x+1, self.y)));
        }
        if self.x > 0 {
            ret.push(Neighbor::Left(Vu2::new(self.x-1, self.y)));
        }
        if self.y > 0 {
            ret.push(Neighbor::Down(Vu2::new(self.x, self.y-1)));
        }
        ret
    }
    pub fn get_neighbors(&self) -> Vec<Neighbor> {
        let mut ret = Vec::new();
        ret.push(Neighbor::Up(Vu2::new(self.x, self.y+1)));
        ret.push(Neighbor::Right(Vu2::new(self.x+1, self.y)));
        if self.x > 0 {
            ret.push(Neighbor::Left(Vu2::new(self.x-1, self.y)));
        }
        if self.y > 0 {
            ret.push(Neighbor::Down(Vu2::new(self.x, self.y-1)));
        }
        ret
    }
    pub fn get_neighbors_vu2(&self) -> Vec<Vu2> {
        let mut ret = Vec::new();
        ret.push(Vu2::new(self.x, self.y+1));
        ret.push(Vu2::new(self.x+1, self.y));
        if self.x > 0 {
            ret.push(Vu2::new(self.x-1, self.y));
        }
        if self.y > 0 {
            ret.push(Vu2::new(self.x, self.y-1));
        }
        ret
    }
}


pub fn get_2d_vec<T: Default>(xlen: usize, ylen: usize) -> Vec<Vec<T>> {
    let mut ret = Vec::new();

    for _ in 0..xlen {
        let mut row = Vec::new();
        for _ in 0..ylen {
            row.push(T::default());
        }
        ret.push(row);
    }
    ret
}

pub fn make_string_at_least_length(mut s: String, target_len: usize, fillchar: char) -> String {
    if s.len() < target_len {
        for _ in s.len()..target_len {
            s.push(fillchar);
        }
        s
    } else {
        s
    }
}
pub fn make_string_at_most_length(mut s: String, target_len: usize) -> String{
    s.truncate(target_len);
    s
}

pub static COUNTER: AtomicU64 = AtomicU64::new(1); // TODO: Upgrade to a 128 bit one when it comes out of nightly build
pub static MAX_ATTACK_DISTANCE: usize = 2;
pub fn get_id() -> u64 { COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }
pub type UID = u64;
pub type BattleFrame = u64;

