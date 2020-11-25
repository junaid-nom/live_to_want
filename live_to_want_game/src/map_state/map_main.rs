use std::{vec::Drain, fmt};

use crate::{UID, creature::CreatureState, creature::IDComponent, get_2d_vec, utils::Vector2, make_string_certain_length};
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
    pub fn find_closest_non_blocked(&self, loc: Location, blocker: bool) -> Option<Location> {
        let region = &self.regions[loc.region.x as usize][loc.region.y as usize];
        let mut to_check: Vec<Vector2> = Vec::new();
        to_check.push(loc.position);
        let mut idx = 0;
        while idx < to_check.len() {
            let checking  = &region.grid[to_check[idx].x as usize][to_check[idx].y as usize];
            if checking.get_if_blocked(blocker) && region.get_if_will_not_cause_blocked_paths(to_check[idx]) {
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
#[derive(Default, PartialEq, Clone)]
pub struct RegionDistances {
    pub left: Option<u32>,
    pub right: Option<u32>,
    pub up: Option<u32>,
    pub down: Option<u32>,
}
impl RegionDistances {
    pub fn new(start: &Vector2, leftv: &Vector2, rightv: &Vector2, upv: &Vector2, downv: &Vector2, region: &MapRegion) -> Self {
        RegionDistances {
            left: match region.get_distance(start, leftv) {
                LocDistance::Unset => {panic!("trying to get region distances from unset region")}
                LocDistance::Blocked => {None}
                LocDistance::Set(d) => {Some(*d)}
            },
            right: match region.get_distance(start, rightv) {
                LocDistance::Unset => {panic!("trying to get region distances from unset region")}
                LocDistance::Blocked => {None}
                LocDistance::Set(d) => {Some(*d)}
            },
            up: match region.get_distance(start, upv) {
                LocDistance::Unset => {panic!("trying to get region distances from unset region")}
                LocDistance::Blocked => {None}
                LocDistance::Set(d) => {Some(*d)}
            },
            down: match region.get_distance(start, downv) {
                LocDistance::Unset => {panic!("trying to get region distances from unset region")}
                LocDistance::Blocked => {None}
                LocDistance::Set(d) => {Some(*d)}
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExitPoint {
    None,
    Left,
    Right,
    Up,
    Down,
    LeftDown,
    RightDown,
    LeftUp,
    RightUp,
}
impl Default for ExitPoint {
    fn default() -> Self {
        ExitPoint::None
    }
    
}
impl fmt::Display for ExitPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let f_string = match &self {
            ExitPoint::None => {""}
            ExitPoint::Left => {"L "}
            ExitPoint::Right => {"R "}
            ExitPoint::Up => {"U "}
            ExitPoint::Down => {"D "}
            ExitPoint::LeftDown => {"LD"}
            ExitPoint::RightDown => {"RD"}
            ExitPoint::LeftUp => {"LU"}
            ExitPoint::RightUp => {"RU"}
        };
        write!(f, "{}", f_string)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LocRegionDistance {
    Unset,
    Set(RegionDistances),
}
impl Default for LocRegionDistance {
    fn default() -> Self {
        LocRegionDistance::Unset
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    pub exists: bool,
    pub grid: Vec<Vec<MapLocation>>,
    pub last_frame_changed: u128, // if nav system last updated before this frame, update it
    // nav stuff:
    pub region_distances: Vec<Vec<LocDistance>>, // cached distance to eveey other region in from this region
    pub distances_from_left: LocRegionDistance,
    pub distances_from_right: LocRegionDistance,
    pub distances_from_up: LocRegionDistance,
    pub distances_from_down: LocRegionDistance,
}
impl fmt::Display for MapRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = Vec::new();
        let line_space = 5;
        let xlen = self.grid.len();
        let ylen = self.grid[0].len();
        for y in 0..ylen {
            let mut f_string = String::new();
            for x in 0..xlen {
                let ml = &self.grid[x][y];
                let dy = if ml.creatures.get_if_blocked() {
                    if ml.is_exit == ExitPoint::None {
                        make_string_certain_length("X".to_string(), line_space, ' ')
                    } else {
                        make_string_certain_length(format!("X{}", ml.is_exit.to_string()), line_space, ' ')
                    }
                } else {
                    if ml.is_exit == ExitPoint::None {
                        make_string_certain_length("O".to_string(), line_space, ' ')
                    } else {
                        make_string_certain_length(format!("O{}", ml.is_exit.to_string()), line_space, ' ')
                    }
                };
                f_string = format!("{}{}", f_string, dy);
                //f_string = format!("{}{}{}_", f_string, ml.location.x, ml.location.y);
            }
            lines.insert(0, f_string);
        }
        
        write!(f, "{}", lines.join("\n"))
    }
}
impl MapRegion {
    pub fn new(xlen: usize, ylen: usize, current_frame: u128, no_creatures: &Vec<Vector2>, has_left_neighbor: bool, has_right_neighbor: bool, has_up_neighbor: bool, has_down_neighbor: bool) -> Self {
        let mut grid: Vec<Vec<MapLocation>> = Vec::new();
        if xlen + ylen >= 2 {
            for x in 0..xlen {
                let mut row = Vec::new();
                for y in 0..ylen {
                    let mut is_exit = ExitPoint::None;
                    let mut is_unblocked_exit = true;
                    if x == 0 {
                        if !has_left_neighbor {
                            is_unblocked_exit = false;
                        }
                        if y == 0 {
                            if !has_down_neighbor {
                                is_unblocked_exit = false;
                            }
                            is_exit = ExitPoint::LeftDown;
                        }
                        else if y == ylen - 1 {
                            if !has_up_neighbor {
                                is_unblocked_exit = false;
                            }
                            is_exit = ExitPoint::LeftUp;
                        }
                        else {
                            is_exit = ExitPoint::Left;
                        }
                    }
                    else if x == xlen - 1 {
                        if !has_right_neighbor {
                            is_unblocked_exit = false;
                        }
                        if y == 0 {
                            if !has_down_neighbor {
                                is_unblocked_exit = false;
                            }
                            is_exit = ExitPoint::RightDown;
                        }
                        else if y == ylen - 1 {
                            if !has_up_neighbor {
                                is_unblocked_exit = false;
                            }
                            is_exit = ExitPoint::RightUp;
                        }
                        else {
                            is_exit = ExitPoint::Right;
                        }
                    }
                    else if y == 0 {
                        if !has_down_neighbor {
                            is_unblocked_exit = false;
                        }
                        is_exit = ExitPoint::Down;
                    }
                    else if y == ylen - 1 {
                        if !has_up_neighbor {
                            is_unblocked_exit = false;
                        }
                        is_exit = ExitPoint::Up;
                    }
                    row.push(MapLocation::new(Vector2::new(x as i32, y as i32), is_exit, is_unblocked_exit, current_frame, xlen, ylen));
                }
                grid.push(row);
            }
            for no in no_creatures {
                let xx = no.x as usize;
                let yy = no.y as usize;
                grid[xx][yy].creatures = CreatureList::new(false, current_frame);
            }
        }

        let mut ret = MapRegion {
            exists: xlen + ylen >= 2,
            grid,
            last_frame_changed: current_frame,
            region_distances: get_2d_vec(xlen, ylen),
            distances_from_left: LocRegionDistance::Unset,
            distances_from_right: LocRegionDistance::Unset,
            distances_from_up: LocRegionDistance::Unset,
            distances_from_down: LocRegionDistance::Unset,
        };
        ret.update_region_nav(current_frame);
        ret
    }

    pub fn copy_blocked(src: &MapRegion) -> Self {
        let mut grid = Vec::new();
        for col in &src.grid {
            let mut new_col = Vec::new();
            for pt in col {
                new_col.push(
                    MapLocation::new(pt.location.clone(), pt.is_exit.clone(), pt.creatures.holds_creatures(), 0, pt.point_distances.len(), pt.point_distances[0].len())
                )
            }
            grid.push(new_col);
        }
        MapRegion {
            exists: src.exists,
            grid,
            last_frame_changed: src.last_frame_changed,
            region_distances: src.region_distances.clone(),
            distances_from_left: src.distances_from_left.clone(),
            distances_from_right: src.distances_from_right.clone(),
            distances_from_up: src.distances_from_up.clone(),
            distances_from_down: src.distances_from_down.clone(),
        }
    }

    pub fn get_distance_strings(&self, end_point: &Vector2) -> Vec<String> {
        let mut lines = Vec::new();
        let xx = end_point.x as usize;
        let yy = end_point.y as usize;
        let xlen = self.grid.len();
        let ylen = self.grid[0].len();
        for y in 0..ylen {
            let mut f_string = String::new();
            for x in 0..xlen {
                let ml = &self.grid[x][y];
                let dy = make_string_certain_length(format!("{}", ml.point_distances[xx][yy]), 5, ' ');
                f_string = format!("{}{}", f_string, dy);
                //f_string = format!("{}{}{}_", f_string, ml.location.x, ml.location.y);
            }
            lines.insert(0, f_string);
        }
        lines
    }

    pub fn get_if_will_not_cause_blocked_paths(&self, loc: Vector2) -> bool {
        // TODONEXT: Calculate if this region will have blocked paths if you place in a location
        true
    }

    pub fn get_distance(&self, start: &Vector2, end: &Vector2) -> &LocDistance {
        &self.grid[start.x as usize][start.y as usize].point_distances[end.x as usize][end.y as usize]
    }

    pub fn update_region_nav(&mut self, current_frame: u128) {
        // TODO: Update all the MapLocation's distances to each other.
        let x_len = self.grid.len();
        let y_len = self.grid[0].len();

        self.grid.par_iter_mut().for_each(|xp| {
            xp.par_iter_mut().for_each(|yp| {
                yp.reset_point_distances();
            })
        });

        let mut up_exit: Option<Vector2> = None;
        let mut down_exit: Option<Vector2> = None;
        let mut right_exit: Option<Vector2> = None;
        let mut left_exit: Option<Vector2> = None;

        let neighbors = vec![Vector2::new(-1, 0), Vector2::new(1, 0), Vector2::new(0, -1), Vector2::new(0, 1)];
        for x in 0..x_len {
            for y in 0..y_len {
                self.grid[x][y].point_distances[x][y] = LocDistance::Set(0);
                //println!("Setting {} {}", x, y);
                // TODO NOTE: this is a really lazy way of getting the exit nodes.
                // so its slightly inaccurate way to get distances between exit points because we just
                let end_blocked = self.grid[x][y].get_if_blocked(false);
                let dist_x_mid = |xd: i32| {
                    (xd - ((x_len/2) as i32)).abs()
                };
                let dist_y_mid = |yd: i32| {
                    (yd - ((y_len/2) as i32)).abs()
                };
                //prioritise mid points for exits for more accurate calculatuon
                let dist_x = dist_x_mid(x as i32);
                let dist_y = dist_y_mid(y as i32);
                if !end_blocked {
                    match self.grid[x][y].is_exit {
                        ExitPoint::None => {}
                        ExitPoint::Left => {
                            if left_exit.is_none() || dist_y < dist_y_mid(left_exit.as_ref().unwrap().y) {
                            left_exit = Some(Vector2::new(x as i32, y as i32));
                        }}
                        ExitPoint::Right => {
                            if right_exit.is_none() || dist_y < dist_y_mid(right_exit.as_ref().unwrap().y) {
                                right_exit = Some(Vector2::new(x as i32, y as i32));
                        }}
                        ExitPoint::Up => {
                            if up_exit.is_none() || dist_x < dist_x_mid(up_exit.as_ref().unwrap().x) {
                                up_exit = Some(Vector2::new(x as i32, y as i32));
                        }}
                        ExitPoint::Down => {
                            if down_exit.is_none() || dist_x < dist_x_mid(down_exit.as_ref().unwrap().x) {
                                down_exit = Some(Vector2::new(x as i32, y as i32));
                        }}
                        ExitPoint::LeftDown => {
                            println!(" LEFT DOWN LEFT DOWN LEFT DOWN");
                            if left_exit.is_none() {left_exit = Some(Vector2::new(x as i32, y as i32));}
                            if down_exit.is_none() {down_exit = Some(Vector2::new(x as i32, y as i32));}
                        }
                        ExitPoint::RightDown => {
                            if right_exit.is_none() {right_exit = Some(Vector2::new(x as i32, y as i32));}
                            if down_exit.is_none() {down_exit = Some(Vector2::new(x as i32, y as i32));}
                        }
                        ExitPoint::LeftUp => {
                            if left_exit.is_none() {left_exit = Some(Vector2::new(x as i32, y as i32));}
                            if up_exit.is_none() {up_exit = Some(Vector2::new(x as i32, y as i32));}
                        }
                        ExitPoint::RightUp => {
                            if right_exit.is_none() {right_exit = Some(Vector2::new(x as i32, y as i32));}
                            if up_exit.is_none() {up_exit = Some(Vector2::new(x as i32, y as i32));}
                        }
                    }
                }
                let mut to_visit: Vec<Vector2> = Vec::new();
                let mut node_idx = 0;
                // add neighbors to, to_visit.
                // then add their neighbors etc.
                for neighbor in &neighbors {
                    to_visit.push(Vector2::add(&Vector2::new(x as i32, y as i32), neighbor));
                }
                //println!("starting neighbors: {:#?}", to_visit);
                while node_idx < to_visit.len() {
                    // for each node, get its neighbor with the lowest distance to target
                    // then set this points distance to 1 + min_neighbor_distance
                    // also add this nodes neighbors to to_visit, if they have an unset point_distance
                    let v = to_visit[node_idx];
                    let xx = v.x;
                    let yy = v.y;
                    //println!("target: {},{} from:{},{}", x,y,xx,yy);
                    if xx < x_len as i32 && xx >= 0 && yy < y_len as i32 && yy >= 0 {
                        let xx = xx as usize;
                        let yy = yy as usize;
                        if self.grid[xx][yy].point_distances[x][y] == LocDistance::Unset {
                            if !self.grid[xx][yy].get_if_blocked(false) {
                                // get neighbor that has point_distance set:
                                let mut min_distance: Option<u32> = None;
                                for neighbor in &neighbors {
                                    let xxn = xx as i32 + neighbor.x;
                                    let yyn = yy as i32 + neighbor.y;
                                    if xxn < x_len as i32 && xxn >= 0 && yyn < y_len as i32 && yyn >= 0 { 
                                        let xxn = xxn as usize;
                                        let yyn = yyn as usize;
                                        match self.grid[xxn][yyn].point_distances[x][y] {
                                            LocDistance::Unset => {
                                                // TODO NOTE: This will add duplicates to the list of already visited places.
                                                // So can either check if to_visit has this var, or just skip in the loop as done above.
                                                // maybe worth changing but probably not.
                                                to_visit.push(Vector2::new(xxn as i32, yyn as i32));
                                            }
                                            LocDistance::Blocked => {}
                                            LocDistance::Set(dist) => {
                                                // pretty sure its impossible for a node to get seen by a slower path so this if is pointless
                                                // if min_distance == 0 || dist <= min_distance {
                                                // }
                                                min_distance = Some(dist);
                                            }
                                        }
                                    }
                                }
                                
                                if !end_blocked {
                                    if let Some(min_distance) = min_distance {
                                        self.grid[xx][yy].point_distances[x][y] = LocDistance::Set(min_distance + 1);
                                    } else {
                                        panic!("Got no neighbor that is in route to destination!");
                                    }
                                } else {
                                    self.grid[xx][yy].point_distances[x][y] = LocDistance::Blocked;
                                }
                            } else {
                                self.grid[xx][yy].point_distances[x][y] = LocDistance::Blocked;
                            }
                        }
                    }
                    node_idx+=1;
                }
                // Anything unset by now must be blocked off
                for xx in 0..x_len {
                    for yy in 0..y_len {
                        if self.grid[xx][yy].point_distances[x][y] == LocDistance::Unset {
                            self.grid[xx][yy].point_distances[x][y] = LocDistance::Blocked;
                        }
                    }
                }
            }
        }
        // also update distances_from_exits
        let vl = Vector2::new(0,y_len as i32/2);
        let leftv = match &left_exit {
            Some(v) => {v}
            None => {&vl}
        };
        let vr = Vector2::new(x_len as i32 - 1,y_len as i32/2);
        let rightv = match &right_exit {
            Some(v) => {v}
            None => {&vr}
        };
        let vu = Vector2::new(x_len as i32/2,y_len as i32 - 1);
        let upv = match &up_exit {
            Some(v) => {v}
            None => {&vu}
        };
        let vd = Vector2::new(x_len as i32/2,0);
        let downv = match &down_exit {
            Some(v) => {v}
            None => {&vd}
        };
        // println!("leftv: {:?}", leftv);
        // println!("{}", self.get_distance_strings(&leftv).join("\n"));
        // println!("rightv: {:?}", rightv);
        // println!("{}", self.get_distance_strings(&rightv).join("\n"));
        // println!("downv: {:?}", downv);
        // println!("{}", self.get_distance_strings(&downv).join("\n"));
        // println!("upv: {:?}", upv);
        // println!("{}", self.get_distance_strings(&upv).join("\n"));

        self.distances_from_left = LocRegionDistance::Set(RegionDistances::new(leftv, leftv, rightv, upv, downv, self));
        self.distances_from_right = LocRegionDistance::Set(RegionDistances::new(rightv, leftv, rightv, upv, downv, self));
        self.distances_from_up = LocRegionDistance::Set(RegionDistances::new(upv, leftv, rightv, upv, downv, self));
        self.distances_from_down = LocRegionDistance::Set(RegionDistances::new(downv, leftv, rightv, upv, downv, self));

        self.last_frame_changed = current_frame;
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LocDistance {
    Unset,
    Blocked,
    Set(u32),
}
impl Default for LocDistance {
    fn default() -> Self {
        LocDistance::Unset
    }
}
impl fmt::Display for LocDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocDistance::Unset => {write!(f, "{}", "U")}
            LocDistance::Blocked => {write!(f, "{}", "X")}
            LocDistance::Set(d) => {write!(f, "{}", d)}
        }
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
    pub point_distances: Vec<Vec<LocDistance>>,
}
impl MapLocation {
    pub fn new(loc: Vector2, is_exit: ExitPoint, has_creatures: bool, current_frame: u128, xlen: usize, ylen: usize) -> Self {
        MapLocation {
            id_component_items: IDComponent::new(),
            id_component_creatures: IDComponent::new(),
            location: loc,
            is_exit,
            creatures: CreatureList::new(has_creatures, current_frame),
            items: Vec::new(),
            point_distances: get_2d_vec(xlen, ylen),
        }
    }

    pub fn get_if_blocked(&self, exits_count_as_blocked: bool) -> bool {
        if self.is_exit != ExitPoint::None && exits_count_as_blocked {
            true
        } else {
            self.creatures.get_if_blocked()
        }
    }
    pub fn reset_point_distances(&mut self) {
        let x_len = self.point_distances.len();
        let y_len = self.point_distances[0].len();
        self.point_distances = Vec::new();
        for x in 0..x_len {
            let mut row = Vec::new();
            for y in 0..y_len {
                row.push(LocDistance::Unset);
            }
            self.point_distances.push(row);
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
