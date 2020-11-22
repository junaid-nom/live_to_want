use std::vec::Drain;

use crate::{UID, creature::CreatureState, creature::IDComponent, utils::Vector2};
use rand::prelude::*;
extern crate rayon;
use rayon::prelude::*;

use super::Item;


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
            if checking.get_if_blocked(true) && region.get_if_will_not_cause_blocked_paths(to_check[idx]) {
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
                return r[position.x as usize][position.y as usize].creatures.holds_creatures()
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

    // nav stuff
    pub fn navigate_to(&mut self, start: &Location, goal: &Location) -> Vec<Location> {
        // Currently just using a simple algo that assumes there are NO blockers anywhere and in same region
        // TODO: make a VecVec VecVec of region(with last updated piece)->location->blocked. and then 
        // make a giant cached navigation thing FOR EACH point...
        // will get weird cause if u change the viable entrance/exits of regions it would mean needing to change the
        // between region map as well.
        // Need to also teach AI how to like "break" things to create shorter path?
        let mut ret = Vec::new();
        if start.region == goal.region {
            let mut current_loc = start.position;
            while current_loc != goal.position {
                let xchange = 
                    if current_loc.x > goal.position.x { -1 } 
                    else if current_loc.x < goal.position.x { 1 }
                    else { 0 };
                let ychange = 
                    if current_loc.y > goal.position.y { -1 } 
                    else if current_loc.y < goal.position.y { 1 }
                    else { 0 };
                if xchange == 0 { current_loc.y += ychange; } else if ychange == 0 { current_loc.x += xchange; } 
                    else {
                        if rand::random() {
                            current_loc.x += xchange;
                        } else {
                            current_loc.y += ychange;
                        }
                    };
                ret.push(Location{region:start.region, position: current_loc});
            }
        } else {
            panic!("Havent implemented cross-region navigation yet");
        }
        ret
    }

    pub fn update_nav(&mut self) {
        // update the region distances for all regions
        
        // if the left/right/up/down access changes then update all the region_distances

        // PANIC if exit nodes are blocked by a creature. also if exit nodes arent together, like there shouldnt be a permablocked location inbetween 2 exit nodes. like for top if it was OOOXOO thats bad because it can cause strange splits where one region is accessible from another but only from a particular entrance. wish I had a better way to make sure u cant do this
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
pub struct RegionDistances {
    left: Option<u32>,
    right: Option<u32>,
    up: Option<u32>,
    down: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub enum ExitPoint {
    None,
    Left,
    Right,
    Up,
    Down,
}
impl Default for ExitPoint {
    fn default() -> Self {
        ExitPoint::None
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    pub grid: Vec<Vec<MapLocation>>,
    pub last_frame_changed: u128, // if nav system last updated before this frame, update it
    // nav stuff:
    pub region_distances: Vec<Vec<u32>>, // cached distance to eveey other region in from this region
    distances_from_left: Option<RegionDistances>,
    distances_from_right: Option<RegionDistances>,
    distances_from_up: Option<RegionDistances>,
    distances_from_down: Option<RegionDistances>,
}
impl MapRegion {
    pub fn get_if_will_not_cause_blocked_paths(&self, loc: Vector2) -> bool {
        // TODO: Calculate if this region will have blocked paths if you place in a location
        true
    }
    pub fn update_region_nav(&mut self, current_frame: u128) {
        // TODO: Update all the MapLocation's distances to each other.
        self.last_frame_changed = current_frame;
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation {
    pub id_component_items: IDComponent,
    pub id_component_creatures: IDComponent,
    pub location: Vector2,
    pub is_exit: ExitPoint, // exits should not be allowed to have creatures placed on them. also they must not have a block INBETWEEN them.
    pub creatures: CreatureList, // some locations will be perma blocked and no creatures allowed
    pub items: Vec<Item>,
    pub point_distances: Vec<Vec<u32>>,
}
impl MapLocation {
    pub fn get_if_blocked(&self, exits_count_as_blocked: bool) -> bool {
        if self.is_exit != ExitPoint::None && exits_count_as_blocked {
            true
        } else {
            self.creatures.get_if_blocked()
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
#[derive(PartialEq, Eq)]
pub struct CreatureList {
    creatures: Option<Vec<CreatureState>>, // some locations will be perma blocked and no creatures allowed so thats None for this
    last_frame_changed: u128,
    blocked: bool,
}
impl CreatureList {
    pub fn new(has_creatures: bool, frame: u128) -> CreatureList {
        CreatureList {
            creatures: if has_creatures {
                Some(Vec::new())
            } else {
                None
            },
            last_frame_changed: frame,
            blocked: !has_creatures,
        }
    }

    pub fn get_last_updated(&self) -> u128 {
        self.last_frame_changed
    }

    fn update_blocked(&mut self, new: bool, current_frame: u128) {
        if self.blocked != new { 
            self.blocked = new;
            self.last_frame_changed = current_frame;
        }
    }

    fn check_and_update_blocked(&mut self, current_frame: u128) {
        match &mut self.creatures {
            Some(creatures) => {
                let mut blocked = false;
                creatures.iter().for_each(|c| {
                    if let Some(_) = c.components.block_space_component {
                        blocked = true;
                    }
                });
                self.update_blocked(blocked, current_frame);
            }
            None => { }
        }
    }

    pub fn add_creature(&mut self, c: CreatureState, current_frame: u128  ) {
        if let Some(_) = c.components.block_space_component {
            self.update_blocked(true, current_frame);
        }
        &self.creatures.as_mut().unwrap().push(c);
    }

    pub fn drain_creatures(&mut self, current_frame: u128) -> Vec<CreatureState> {
        let old_len = self.creatures.as_ref().unwrap().len();
        let mut new_creatures = Some(Vec::new());
        std::mem::swap(&mut self.creatures, &mut new_creatures);

        let cmut = new_creatures.unwrap();
        self.update_blocked(false, current_frame);
        assert_eq!(cmut.len(), old_len);
        assert_eq!(self.creatures.as_ref().unwrap().len(), 0);
        cmut
    }

    pub fn drain_specific_creature(&mut self, id: UID, current_frame: u128) -> CreatureState {
        let to_rm = self.creatures.as_ref().unwrap().iter().position(|c: &CreatureState| {
            c.components.id_component.id() != id
        }).unwrap();
        let rmed = self.creatures.as_mut().unwrap().remove(to_rm);
        if let Some(_) = rmed.components.block_space_component {
            // TODO: Not sure if this could be inaccurate cause maybe there are 2 blockers there?
            self.update_blocked(false, current_frame);
        }
        rmed
    }

    pub fn get_if_blocked(&self) -> bool {
        if let Some(creatures) = self.creatures.as_ref() {
            for c in creatures.iter() {
                if let Some(_) = c.components.block_space_component {
                    return true
                }
            }
        } else {
            return true;
        }
        
        return false;
    }
    pub fn holds_creatures(&self) -> bool {
        match self.creatures {
            Some(_) => { true }
            None => { false }
        }
    }
    pub fn get_par_iter_mut(&mut self) -> Option<rayon::slice::IterMut<'_, CreatureState>>{
        match &mut self.creatures {
            Some(creatures) => { Some(creatures.par_iter_mut()) }
            None => { None }
        }
    }
    pub fn get_par_iter(&self) -> Option<rayon::slice::Iter<'_, CreatureState>>{
        match &self.creatures {
            Some(creatures) => { Some(creatures.par_iter()) }
            None => { None }
        }
    }
    pub fn get_iter_mut(&mut self) -> Option<std::slice::IterMut<'_, CreatureState>>{
        match &mut self.creatures {
            Some(creatures) => { Some(creatures.iter_mut()) }
            None => { None }
        }
    }

    pub fn drain_all_but_first_blocker(&mut self, current_frame: u128) -> (Vec<CreatureState>, Vec<CreatureState>) {
        let mut ret: (Vec<CreatureState>, Vec<CreatureState>) = (Vec::new(), Vec::new());
        if let Some(creatures) = self.creatures.as_mut() {
            // if there is a blocking creature and any other creature here
            // then we have to remove them
            let mut first_blocker: Option<UID> = None;
            for i in 0..creatures.len() {
                let c = &creatures[i];
                if let Some(_) = c.components.block_space_component {
                    first_blocker = Some(c.components.id_component.id());
                    break;
                }
            };
            if let Some(first) = first_blocker {
                for i in 0..creatures.len() {
                    if i < creatures.len() {
                        let c = &creatures[i];
                        if c.components.id_component.id() != first {
                            if let Some(_) = c.components.block_space_component {
                                ret.0.push(creatures.remove(i));
                            } else {
                                ret.1.push(creatures.remove(i));
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        self.check_and_update_blocked(current_frame);
        return ret;
    }

    pub fn drain_no_health(&mut self, current_frame: u128) -> Vec<CreatureState> {
        let creatures = self.creatures.as_mut().unwrap();
        let mut i = 0;
        fn is_dead(c: &CreatureState) -> bool {
            if let Some(h) = c.components.health_component.as_ref() {
                if h.health <= 0 {
                    false
                } else {
                    true
                }
            } else {
                true
            }
        }
        let mut ret = Vec::new();
        while i != creatures.len() {
            if is_dead(&creatures[i]) {
                let val = creatures.remove(i);
                ret.push(val);
            } else {
                i += 1;
            }
        }
        self.check_and_update_blocked(current_frame);
        return ret;
        // One day drain_filter wont be on nightly
        // self.creatures.as_mut().unwrap().drain_filter(|c| {
        //     if let Some(h) = c.components.health_component.as_ref() {
        //         if h.health <= 0 {
        //             false
        //         } else {
        //             true
        //         }
        //     } else {
        //         true
        //     }
        // })
    }
}
