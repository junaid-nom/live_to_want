// game constants:
static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;
static REPRODUCE_STARTING_CALORIES: i32 = 150;

trait Component {
    fn get_visible() -> bool {
        false
    }
}

#[derive(Default)]
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct ComponentMap {
    id_component: IDComponent,
    health_component: Option<HealthComponent>,
    location_component: LocationComponent,
    region_component: RegionComponent,
    name_component: Option<NameComponent>,
    creature_type_component: Option<CreatureTypeComponent>,
    starvation_component: Option<StarvationComponent>,
    block_space_component: Option<BlockSpaceComponent>,
    movement_component: Option<MovementComponent>,
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
struct IDComponent {
    id: UID,
}
impl IDComponent {
    fn new() -> IDComponent{
        IDComponent{
            id: get_id()
        }
    }
}
impl Default for IDComponent {
    fn default() -> Self {
        IDComponent::new()
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
struct HealthComponent {
    health: i32,
    max_health: i32,
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
struct BuddingComponent {
    frame_ready_to_reproduce: u128,
    seed_creature: CreatureState,
}
impl Component for BuddingComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
struct LocationComponent {
    location: Vector2,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct RegionComponent {
    region: Vector2,
}
impl Component for RegionComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct NameComponent {

}
impl Component for NameComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct CreatureTypeComponent {

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
struct BlockSpaceComponent {
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
    speed: usize,
    destination: Location,
    cached_navigation: Vec<Location>,
    cache_last_updated_frame: u128,
    navigating: bool,
    moving: bool,
    frame_ready_to_move: u128, // essentially if frame_ready to move is the current frame or earlier, move to destination
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
struct StarvationComponent {
    calories: i32,
    metabolism: usize,
}
impl Component for StarvationComponent {
    fn get_visible() -> bool {
        true
    }
}
