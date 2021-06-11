use crate::{Item, map_state::Location, utils::{UID, get_id, Vector2, Vu2}};

use super::CreatureState;

// game constants:
pub static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;
pub static REPRODUCE_STARTING_CALORIES: i32 = 150;

pub trait Component: Sync + Send + Clone {
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
    pub soil_component: Option<SoilComponent>, // used by budding system (and maybe others in future?)
    pub death_items_component: Option<DeathItemsComponent>,// has items that ARE NOT in inventory that should be dropped on death. for example antlers for a deer
    pub battle_component: Option<BattleComponent>,
} 
impl ComponentMap {
    pub fn copy_from_other(self, other:&ComponentMap) -> Self {
        let other = other.fake_clone();
        ComponentMap {
            id_component: IDComponent {
                id: self.id_component.id
            },
            health_component: other.health_component.or(self.health_component),
            location_component: self.location_component,
            region_component: self.region_component,
            name_component: other.name_component.or(self.name_component),
            creature_type_component: other.creature_type_component.or(self.creature_type_component),
            starvation_component: other.starvation_component.or(self.starvation_component),
            block_space_component: other.block_space_component.or(self.block_space_component),
            movement_component: other.movement_component.or(self.movement_component),
            budding_component: other.budding_component.or(self.budding_component),
            soil_component: other.soil_component.or(self.soil_component),
            death_items_component: other.death_items_component.or(self.death_items_component),
            battle_component: other.battle_component.or(self.battle_component),
        }
    }
    // Only meant to be used for budding component and similar. Should never put a fake_clone onto a real creature because no UID
    pub fn fake_clone(&self) -> Self {
        ComponentMap {
            id_component: IDComponent::fake_clone(),
            health_component: self.health_component.clone(),
            location_component: self.location_component.clone(),
            region_component: self.region_component.clone(),
            name_component: self.name_component.clone(),
            creature_type_component: self.creature_type_component.clone(),
            starvation_component: self.starvation_component.clone(),
            block_space_component: self.block_space_component.clone(),
            movement_component: self.movement_component.clone(),
            budding_component: self.budding_component.clone(),
            soil_component: self.soil_component.clone(),
            death_items_component: self.death_items_component.clone(),
            battle_component: self.battle_component.clone(),
        }
    }
    pub fn fake_default() -> Self {
        ComponentMap {
            id_component: IDComponent::fake_clone(),
            health_component: None,
            location_component: LocationComponent::default(),
            region_component: RegionComponent::default(),
            name_component: None,
            creature_type_component: None,
            starvation_component: None,
            block_space_component: None,
            movement_component: None,
            budding_component: None,
            soil_component: None,
            death_items_component: None,
            battle_component: None,
        }
    }
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
    fn fake_clone() -> Self {
        // SHOULD ONLY BE USED FOR BUDDING. Should never be actually placed on a creature state!
        IDComponent {
            id: 0,
        }
    }
}
impl Default for IDComponent {
    fn default() -> Self {
        IDComponent::new()
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct HealthComponent {
    pub health: i32,
    pub max_health: i32,
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Clone for HealthComponent {
    fn clone(&self) -> Self {
        HealthComponent {
            health: self.max_health,
            max_health: self.max_health
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
// Essentially for budding and plant reproduction, don't reproduce onto a tile that already has something with the uses the same soil layer.
// For things that "bud" that don't need any soil remove the soil component, like if I ever make budding animals
// All Type takes up all the soil nothing can grow EXCEPT Free.
pub enum SoilLayer {
    Grass,
    Flower,
    Bush,
    // Tree would just be something with no soil layber but is a blocker basically
    All, // blocks all growth
}
impl Default for SoilLayer {
    fn default() -> Self { SoilLayer::Grass }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct BuddingComponent {
    pub reproduction_rate: u32,
    pub frame_ready_to_reproduce: u128,
    pub seed_creature_differences: Box<ComponentMap>,
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
            seed_creature_differences: Box::new(self.seed_creature_differences.fake_clone()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(Hash, PartialEq, Eq)]
pub struct SoilComponent {
    pub soil_layer: SoilLayer,
}
impl Component for SoilComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone)]
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
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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
impl Clone for StarvationComponent {
    fn clone(&self) -> Self {
        StarvationComponent{
            calories: self.metabolism as i32 * REPRODUCE_STARTING_CALORIES,
            metabolism: self.metabolism,
        }
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
pub struct BattleComponent {
    pub in_battle:Option<UID>,
    // TODO: Add a GoalNode where if in combat, return None command.
}
impl BattleComponent {
    pub fn add_in_battle(&mut self, battle_id: UID) {
        self.in_battle = Some(battle_id);
    }
    pub fn leave_in_battle(&mut self) {
        self.in_battle = None;
    }
}
impl Component for BattleComponent {
    fn get_visible() -> bool {
        true
    }
}
