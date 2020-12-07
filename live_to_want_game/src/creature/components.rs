use crate::{Item, map_state::Location, utils::{UID, get_id, Vector2, Vu2}};

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
    pub budding_component: Option<BuddingComponent>,
    pub death_items_component: Option<DeathItemsComponent>,
    pub battle_component: Option<BattleComponent>,
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
    pub reproduction_rate: u32,
    pub frame_ready_to_reproduce: u128,
    pub seed_creature: Box<CreatureState>,
}
impl Component for BuddingComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Clone for BuddingComponent {
    fn clone(&self) -> Self {
        BuddingComponent {
            reproduction_rate: self.reproduction_rate,
            frame_ready_to_reproduce: self.frame_ready_to_reproduce + self.reproduction_rate as u128,
            seed_creature: Box::new(CreatureState::copy(self.seed_creature.as_ref(), self.seed_creature.components.location_component.location)),
        }
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct LocationComponent {
    pub location: Vu2,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct RegionComponent {
    pub region: Vu2,
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

// DONE: Okay so I need to CHANGE the event-chain system to also be able to RETURN
// a new event chain dynamically created! That has a new Event type that OWNS a creaturestate
// that is then meant to be moved into a new location (spawn will be similar)
#[derive(Hash, Debug, PartialEq, Eq)]
pub struct MovementComponent {
    //TODO: Add support for "sprint" feature. Also add hgher metabolism thing
    pub frames_to_move: usize,
    pub destination: Location,
    pub frame_ready_to_move: u128, // essentially if frame_ready to move is the current frame or earlier, move to destination
    pub moving: bool,
}
impl MovementComponent {
    pub fn set_new_destination(&mut self, dst:Location, current_frame: u128) {
        //TODONEXT 
        // if already moving right now, and its to the same place allow it.
        if self.moving {
            // fuck it, if your moving keep your frame_ready_to_move
            self.destination = dst;
        } else {
            self.moving = true;
            self.frame_ready_to_move = current_frame + self.frames_to_move as u128;
            self.destination = dst;
        }
    }
    pub fn check_ready_and_reset_move(&mut self, current_frame: u128, dst_reached: bool) -> bool {
        if dst_reached {
            self.moving = false;
        }
        if self.moving && self.frame_ready_to_move <= current_frame{
            // Check if its ready to move, then move and set next frame_ready to move
            self.frame_ready_to_move = current_frame + self.frames_to_move as u128;
            true
        } else {
            false
        }
    }
}
impl Component for MovementComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Clone for MovementComponent {
    fn clone(&self) -> Self {
        MovementComponent{
            frames_to_move: self.frames_to_move,
            destination: self.destination,
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


#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct DeathItemsComponent {
    pub items_to_drop: Vec<Item>,
}
impl Component for DeathItemsComponent {
    fn get_visible() -> bool {
        true
    }
}


#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum StatusEffect {
    NoEscape(u32),
    AtRange(u32),//if a character has this on, any melee attack takes longer to do
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum CombatAI {
    Random,
    Simulator(u32),
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum BattleSkill {
    BiteLeg,
    BiteNeck,
    RunAway,
}// TODO: Make a struct called "BattleSkillAttributes" that store stuff like if an attack is melee or not
// TODO: Will eventually have item-skills. battleskills available when equiping certain items such a sword or gun etc.

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct BattleComponent {
    pub battle_skills: Vec<Item>,
    pub in_battle_with:Option<u128>,
    pub status_effects: Vec<StatusEffect>,
    // TODO: Add a GoalNode where if in combat, return None command.
    // todo put battle only stats here:

}
impl Component for BattleComponent {
    fn get_visible() -> bool {
        true
    }
}
