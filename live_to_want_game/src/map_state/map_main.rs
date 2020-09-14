
#[derive(Debug)]
#[derive(Default)]
pub struct MapState {
    pub regions: Vec<Vec<MapRegion>>,
    pub frame_count: u128,
}

#[derive(Debug)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub region: Vector2,
    pub location: Vector2,
}


#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    grid: Vec<Vec<MapLocation>>,
    last_frame_changed: u128, // if nav system last updated before this frame, update it
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation {
    id_component_items: IDComponent,
    id_component_creatures: IDComponent,
    location: Vector2,
    is_exit: bool, // exits should not be allowed to have creatures placed on them. also they must not have a block INBETWEEN them.
    creatures: Option<Vec<CreatureState>>, // some locations will be perma blocked and no creatures allowed
    items: Vec<Item>,
}
impl MapLocation {
    fn get_if_blocked(&self, target_is_blocker: bool) -> bool {
        if self.is_exit && target_is_blocker {
            return true;
        }
        if let Some(creatures) = self.creatures.as_ref() {
            for c in creatures.iter() {
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

pub fn location_to_map_location<'a>(m: &'a MapState, location: &Location) -> &'a MapLocation {
    let region: &MapRegion = &m.regions[location.region.x as usize][location.region.y as usize];
    &region.grid[location.location.x as usize][location.location.y as usize]
}
