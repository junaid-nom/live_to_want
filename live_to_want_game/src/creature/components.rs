use crate::{Item, map_state::Location, utils::{UID, get_id, Vector2, Vu2}};
use serde::{Deserialize, Serialize};
use super::CreatureState;
use rand::Rng;
// game constants:
pub static STANDARD_HP: i32 = 10000;
pub static SIMPLE_ATTACK_BASE_DMG: i32 = 1000;

pub static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;
pub static REPRODUCE_STARTING_CALORIES: i32 = 150;
pub static MOVING_INCREASED_METABOLISM_FACTOR: f32 = 1.5;

pub static MUTATION_CHANGE: i32 = 5;

pub static THICK_HIDE_METABOLISM_MULTIPLIER: f32 = 0.2 / 100.0;
pub static THICK_HIDE_DMG_REDUCE_MULTIPLIER: f32 = SIMPLE_ATTACK_BASE_DMG as f32 * 1.0 / 100.0; // For every 100 thick hide, decrease dmg by 1

pub static SHARP_CLAWS_DMG_INCREASE: f32 = SIMPLE_ATTACK_BASE_DMG as f32 * 0.7 / 100.0; // for every 100 sharp claws, increase dmg by 1.7x simple attack (rounds down)

pub trait Component: Sync + Send + Clone {
    fn get_visible() -> bool {
        false
    }
}

#[derive(Default, Clone)]
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
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
    pub user_component: Option<UserComponent>,
    pub evolving_traits: Option<EvolvingTraits>,
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
            user_component: other.user_component.or(self.user_component),
            evolving_traits: other.evolving_traits.or(self.evolving_traits),
        }
    }
    // Only meant to be used for budding component and similar. Should never put a fake_clone onto a real creature because no UID
    pub fn fake_clone(&self) -> Self {
        ComponentMap {
            id_component: IDComponent::fake_clone(),
            health_component: self.health_component.as_ref().map_or(None, |hc| Some(hc.fake_clone())),
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
            user_component: self.user_component.clone(),
            evolving_traits: self.evolving_traits.clone(),
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
            user_component: None,
            evolving_traits: None,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
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
#[derive(Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct HealthComponent {
    pub health: i32,
    pub max_health: i32,
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}
impl HealthComponent {
    pub fn fake_clone(&self) -> Self {
        HealthComponent {
            health: self.max_health,
            max_health: self.max_health
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
pub struct SoilComponent {
    pub soil_layer: SoilLayer,
}
impl Component for SoilComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct LocationComponent {
    pub location: Vu2,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct UserComponent {
    pub username: String,
}
impl Component for UserComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[derive(Deserialize, Serialize)]
pub struct RegionComponent {
    pub region: Vu2,
}
impl Component for RegionComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct NameComponent {

}
impl Component for NameComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[derive(Deserialize, Serialize)]
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

#[derive(Debug)]
pub enum Mutations{
    ThickHide(),
    SharpClaws(),
    // Don't include until implemented:
    //Hamstring(),
}

// to allow AI to eventually get smart enough to figure out that they should
// chop down trees to make navigation easier gonna add a "breakable" bool here.
// ACTUALLY I wont cause that should be based on like health component existing
// or something like that
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[derive(Deserialize, Serialize)]
pub struct EvolvingTraits {
    //Implemented: 
    pub thick_hide: i32, // Reduces damage taken by a flat amount? increases calories per movement?
    pub sharp_claws: i32, // Increase damage done by attacksimple

    //Unimplemented:
    pub hamstring: i32, // lowers speed of victim after attacksimple?

}
impl Component for EvolvingTraits {
    fn get_visible() -> bool {
        true
    }
}
impl EvolvingTraits {
    pub fn get_total_metabolism_multiplier(&self, is_moving: bool) -> f32 {
        let mut total: f32 = 1.0;
        total += self.thick_hide as f32 * THICK_HIDE_METABOLISM_MULTIPLIER;
        if is_moving {
            total += MOVING_INCREASED_METABOLISM_FACTOR;
        }
        total
    }

    pub fn get_mutated(&self, mutations: u32) -> EvolvingTraits {
        let mut child = self.clone();

        let mut rng = rand::thread_rng();
        (0..mutations).for_each(|_| {
            let change: i32 = if rng.gen_bool(0.5) {
                MUTATION_CHANGE
            } else {
                -1 * MUTATION_CHANGE
            };
            let chosen = rng.gen_range(0, 2);
            match chosen {
                0 => {
                    child.thick_hide += change;
                },
                1 => {
                    child.sharp_claws += change;
                },
                _ => {
                    panic!("Got to an unimplemented mutation");
                }
            }
        });
        child
    }


    // TODONEXT actually use the below functions in the simple_attack creature command code
    pub fn get_total_simple_attack_adder(&self) -> i32 {
        let mut total = SIMPLE_ATTACK_BASE_DMG;
        total += (self.sharp_claws as f32 * SHARP_CLAWS_DMG_INCREASE).floor() as i32;
        total
    }

    pub fn get_total_defense_subtractor(&self) -> i32 {
        let mut total = 0;
        total += (self.thick_hide as f32 * THICK_HIDE_DMG_REDUCE_MULTIPLIER).floor() as i32;
        total
    }
}


// to allow AI to eventually get smart enough to figure out that they should
// chop down trees to make navigation easier gonna add a "breakable" bool here.
// ACTUALLY I wont cause that should be based on like health component existing
// or something like that
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
pub struct MovementComponent {
    //TODO: Add support for "sprint" feature. Also add hgher metabolism thing
    pub frames_to_move: usize, // this is basically the speed stat but inverted
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
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
