use crate::utils::Vector2;

use super::{MapRegion, Location};

#[derive(Debug)]
#[derive(Default)]
pub struct NavRegion {
    grid: Vec<Vec<NavPoint>>,
    region_distances: Vec<Vec<u32>>,
    last_frame_updated: u128,
    left: bool,
    right: bool,
    up: bool, // TODO: This kinda doesnt make sense maybe need dist between regions idk?
    down: bool,
}

#[derive(Debug)]
#[derive(Default)]
pub struct NavPoint {
    blocked: bool,
    point_distances: Vec<Vec<u32>>,
    is_exit: ExitPoint
}

#[derive(Debug)]
enum ExitPoint {
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
pub struct NavigationMap {
    map: Vec<Vec<NavRegion>>,
}
impl NavigationMap {
    fn update(&mut self, region: Vector2, map_region: &MapRegion) {
        // update the navRegion
        
        // if the left/right/up/down access changes then update all the region_distances

        // PANIC if exit nodes are blocked by a creature. also if exit nodes arent together, like there shouldnt be a permablocked location inbetween 2 exit nodes. like for top if it was OOOXOO thats bad because it can cause strange splits where one region is accessible from another but only from a particular entrance. wish I had a better way to make sure u cant do this
    }

    fn navigate_to(&mut self, start: &Location, goal: &Location) -> Vec<Location> {
        // Currently just using a simple algo that assumes there are NO blockers anywhere and in same region
        // TODO: make a VecVec VecVec of region(with last updated piece)->location->blocked. and then 
        // make a giant cached navigation thing FOR EACH point...
        // will get weird cause if u change the viable entrance/exits of regions it would mean needing to change the
        // between region map as well.
        // Need to also teach AI how to like "break" things to create shorter path?
        let mut ret = Vec::new();
        if start.region == goal.region {
            let mut current_loc = start.location;
            while current_loc != goal.location {
                let xchange = 
                    if current_loc.x > goal.location.x { -1 } 
                    else if current_loc.x < goal.location.x { 1 }
                    else { 0 };
                let ychange = 
                    if current_loc.y > goal.location.y { -1 } 
                    else if current_loc.y < goal.location.y { 1 }
                    else { 0 };
                if xchange == 0 { current_loc.y += ychange; } else if ychange == 0 { current_loc.x += xchange; } 
                    else {
                        if rand::random() {
                            current_loc.x += xchange;
                        } else {
                            current_loc.y += ychange;
                        }
                    };
                ret.push(Location{region:start.region, location: current_loc});
            }
        } else {
            panic!("Havent implemented cross-region navigation yet");
        }
        ret
    }
}
