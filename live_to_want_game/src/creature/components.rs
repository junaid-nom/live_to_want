use crate::{utils::{UID, get_id, Vector2}, map_state::Location};

use super::CreatureState;

// game constants:
pub static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;
pub static REPRODUCE_STARTING_CALORIES: i32 = 150;

pub trait Component {
    fn get_visible() -> bool {
        false
    }
}

#[derive(Default)]
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct ComponentMap {
    pub id_component: IDComponent,
    pub health_component: Option<HealthComponent>,
    pub location_component: LocationComponent,
    pub region_component: RegionComponent,
    pub name_component: Option<NameComponent>,
    pub creature_type_component: Option<CreatureTypeComponent>,
    pub starvation_component: Option<StarvationComponent>,
    pub block_space_component: Option<BlockSpaceComponent>,
    pub movement_component: Option<MovementComponent>,
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct IDComponent {
    id: UID,
}
impl IDComponent {
    pub fn new() -> IDComponent{
        IDComponent{
            id: get_id()
        }
    }
    pub fn id(&self) -> UID {
       self.id 
    }
}
impl Default for IDComponent {
    fn default() -> Self {
        IDComponent::new()
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct HealthComponent {
    pub health: i32,
    pub max_health: i32,
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct BuddingComponent {
    pub frame_ready_to_reproduce: u128,
    pub seed_creature: CreatureState,
}
impl Component for BuddingComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct LocationComponent {
    pub location: Vector2,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct RegionComponent {
    pub region: Vector2,
}
impl Component for RegionComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct NameComponent {

}
impl Component for NameComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct CreatureTypeComponent {

}
impl Component for CreatureTypeComponent {
    fn get_visible() -> bool {
        true
    }
}

// TODO: Either:
// 1. If you have BlocKSpaceComponent you CANNOT have a move component
// or 2. Blockers can move, but there needs to be a special EXTRA entire loop
// check in run_frame where any collisions with blocks and other creatures, the 
// other creatures have to be moved the nearest open space, and if the colliding
// creature is a blocker as well then it has to move to an unoccupied space? and this must be done LINEARLY
// because u cud have 2-4 blockers all moving to the same space

// to allow AI to eventually get smart enough to figure out that they should
// chop down trees to make navigation easier gonna add a "breakable" bool here.
// ACTUALLY I wont cause that should be based on like health component existing
// or something like that
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct BlockSpaceComponent {
}
impl Component for BlockSpaceComponent {
    fn get_visible() -> bool {
        true
    }
}

// Events set the Navigation Component.
// Nagivation system then will set this MovementComponent system
// Movement system will then run, where it will check frame_ready_to_move and if
// its ready, create an event chain to move obj
// DONE: Okay so I need to CHANGE the event-chain system to also be able to RETURN
// a new event chain dynamically created! That has a new Event type that OWNS a creaturestate
// that is then meant to be moved into a new location (spawn will be similar)
#[derive(Hash, Debug, PartialEq, Eq)]
pub struct MovementComponent {
    pub speed: usize,
    pub destination: Location,
    pub cached_navigation: Vec<Location>,
    pub cache_last_updated_frame: u128,
    pub navigating: bool,
    pub moving: bool,
    pub frame_ready_to_move: u128, // essentially if frame_ready to move is the current frame or earlier, move to destination
}
impl Component for MovementComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Clone for MovementComponent {
    fn clone(&self) -> Self {
        MovementComponent{
            speed: self.speed,
            destination: self.destination,
            cached_navigation: Vec::new(),
            cache_last_updated_frame: self.cache_last_updated_frame,
            navigating: false,
            moving: false,
            frame_ready_to_move: self.frame_ready_to_move,
        }
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct StarvationComponent {
    pub calories: i32,
    pub metabolism: usize,
}
impl Component for StarvationComponent {
    fn get_visible() -> bool {
        true
    }
}
