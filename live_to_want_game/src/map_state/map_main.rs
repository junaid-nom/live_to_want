use crate::{utils::Vector2, creature::IDComponent, creature::CreatureState};

use super::Item;
extern crate rayon;
use rayon::prelude::*;


#[derive(Debug)]
#[derive(Default)]
pub struct MapState {
    pub regions: Vec<Vec<MapRegion>>,
    pub frame_count: u128,
}

impl MapState {
    pub fn find_closest_non_blocked(&self, loc: Location) -> Option<Location> {
        let region = &self.regions[loc.region.x as usize][loc.region.y as usize];
        let mut to_check: Vec<Vector2> = Vec::new();
        to_check.push(loc.position);
        let mut idx = 0;
        while idx < to_check.len() {
            let checking  = &region.grid[to_check[idx].x as usize][to_check[idx].y as usize];
            if checking.get_if_blocked(true) {
                // add vector2s to to_check of locations next to this one if they exist
                // and if they aren't already in the list
                let neighbors = to_check[idx].get_neighbors(false);
                for n in neighbors {
                    if self.location_exists_and_holds_creatures(&loc.region, &to_check[idx]) && !to_check.contains(&n) {
                        to_check.push(n);
                    }
                }
            } else {
                return Some(Location {
                    region: loc.region, 
                    position: to_check[idx],
                });
            }
            idx += 1;
        }
        None
    }

    pub fn location_exists_and_holds_creatures(&self, region: &Vector2, position: &Vector2) -> bool {
        if self.regions.len() < region.x as usize && self.regions[region.x as usize].len() < region.y as usize { 
            let r = &self.regions[region.x as usize][region.y as usize].grid;
            if r.len() < position.x as usize && r[position.x as usize].len() < position.y as usize {
                if let Some(_) = r[position.x as usize][position.y as usize].creatures {
                    return true;
                }
            }
        }
        false
    }

    pub fn location_to_map_location<'a>(&'a self, location: &Location) -> &'a MapLocation {
        let region: &MapRegion = &self.regions[location.region.x as usize][location.region.y as usize];
        &region.grid[location.position.x as usize][location.position.y as usize]
    }

    pub fn location_to_map_region<'a>(&'a self, location: &Location) -> &'a MapRegion {
        let region: &MapRegion = &self.regions[location.region.x as usize][location.region.y as usize];
        region
    }
    pub fn vector2_to_map_region<'a>(&'a self, region: &Vector2) -> &'a MapRegion {
        let region = &self.regions[region.x as usize][region.y as usize];
        region
    }
}

#[derive(Debug)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub region: Vector2,
    pub position: Vector2,
}
impl Location{
    pub fn new(region: Vector2, position: Vector2) -> Location {
        Location{
            region,
            position
        }
    }
}


#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    pub grid: Vec<Vec<MapLocation>>,
    pub last_frame_changed: u128, // if nav system last updated before this frame, update it
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation {
    pub id_component_items: IDComponent,
    pub id_component_creatures: IDComponent,
    pub location: Vector2,
    pub is_exit: bool, // exits should not be allowed to have creatures placed on them. also they must not have a block INBETWEEN them.
    pub creatures: Option<CreatureList>, // some locations will be perma blocked and no creatures allowed
    pub items: Vec<Item>,
}
impl MapLocation {
    pub fn get_if_blocked(&self, exits_count_as_blocked: bool) -> bool {
        if self.is_exit && exits_count_as_blocked {
            return true;
        }
        if let Some(creatures) = self.creatures.as_ref() {
            for c in creatures.get_creatures().iter() {
                if let Some(_) = c.components.block_space_component {
                    return true
                }
            }
        } else {
            return true;
        }
        
        false
    }
}


#[derive(Debug)]
#[derive(Default)]
pub struct CreatureList {
    creatures: Vec<CreatureState>,
    is_blocked_previously: bool,
    last_frame_blockers_changed: u128,
}
impl CreatureList {
    pub fn new() -> CreatureList {
        CreatureList {
            creatures: Vec::new(),
            is_blocked_previously: false,
            last_frame_blockers_changed:0,
        }
    }

    pub fn get_creatures(&self) -> &Vec<CreatureState> {
        &self.creatures
    }
    pub fn take_all_creatures(&mut self) -> Vec<CreatureState> {
        let mut creatures_ret = Vec::new();
        std::mem::swap(&mut self.creatures, &mut creatures_ret);
        creatures_ret
    }
    pub fn overwrite_creatures(&mut self, new_vec :Vec<CreatureState>, current_frame: u128) {
        self.creatures = new_vec;
        let mut blocked_now = false;
        for c in &mut self.creatures {
            if c.components.block_space_component.is_some() {
                blocked_now = true;
                break;
            }
        }
        if self.is_blocked_previously != blocked_now {
            self.last_frame_blockers_changed = current_frame;
        }
        self.is_blocked_previously = blocked_now;
    }
    pub fn add_creature(&mut self, new_creature: CreatureState, current_frame: u128) {
        let is_blocking = new_creature.components.block_space_component.is_some();
        if is_blocking && !self.is_blocked_previously {
            self.last_frame_blockers_changed = current_frame;
            self.is_blocked_previously = true;
        }
        
        self.creatures.push(new_creature);
    }
    pub fn get_par_iter_mut(&mut self) -> rayon::slice::IterMut<'_, CreatureState> {
        self.creatures.par_iter_mut()
    }
    pub fn get_par_iter(&self) -> rayon::slice::Iter<'_, CreatureState> {
        self.creatures.par_iter()
    }
    pub fn get_drain(&mut self) -> std::vec::Drain<'_, CreatureState> {
        self.creatures.drain(..)
    }
    pub fn retain(&mut self, func: fn(&CreatureState) -> bool) {
        self.creatures.retain(func);
    }
    pub fn get_iter_mut(&mut self) -> std::slice::IterMut<'_, CreatureState>{
        self.creatures.iter_mut()
    }
    pub fn get_iter(&self) -> std::slice::Iter<'_, CreatureState> {
        self.creatures.iter()
    }
    pub fn get_length(&self) -> usize {
        self.creatures.len()
    }
    //TODO remove creature?
}

