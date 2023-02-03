use crate::{Item, map_state::Location, utils::{UID, get_id, Vu2}};
use serde::{Deserialize, Serialize};
use rand::Rng;
// game constants:
pub static DEBUG_EVENTS: bool = true;

pub static STANDARD_HP: i32 = 10000;
pub static SIMPLE_ATTACK_BASE_DMG: i32 = 1000;
pub static STANDARD_FRAMES_TO_MOVE: i32 = 10;
pub static STANDARD_PREGNANCY_TIME: u128 = 20 * STANDARD_FRAMES_TO_MOVE as u128;
pub static STANDARD_CHILD_TIME: u128 = STANDARD_PREGNANCY_TIME * 2 as u128; // Should be atleast double preg time to make maxing out pregnancy worthwhile?
pub static BASE_PREGNANCY_CHANCE_WEIGHT: i32 = 100;
pub static STANDARD_METABOLISM: i32 = 100;
pub static STANDARD_PREGNANCY_LIVE_WEIGHT: i32 = 100;
pub static STANDARD_PREGNANCY_METABOLISM_MULTIPLIER: f32 = 1.3;

pub static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;
pub static REPRODUCE_STARTING_CALORIES_MULTIPLIER: i32 = 150;
pub static MOVING_INCREASED_METABOLISM_FACTOR: f32 = 1.5;

pub static MUTATION_CHANGE: i32 = 10;
pub static SPECIES_SEX_RANGE: i32 = MUTATION_CHANGE * 5;// Shouldnt be too high because the expected value of species changing is 0, and its rarely mutated.

pub static THICK_HIDE_METABOLISM_MULTIPLIER: f32 = 0.2 / 100.0;
pub static THICK_HIDE_DMG_REDUCE_MULTIPLIER: f32 = SIMPLE_ATTACK_BASE_DMG as f32 * 1.0 / 100.0; // For every 100 thick hide, decrease dmg by 1

pub static SHARP_CLAWS_DMG_INCREASE: f32 = SIMPLE_ATTACK_BASE_DMG as f32 * 0.7 / 100.0; // for every 100 sharp claws, increase dmg by 1.7x simple attack (rounds down)

pub static GIRTH_HEALTH_INCREASE: f32 = (STANDARD_HP as f32 * 0.7) / 100.0;

pub static MOVE_SPEED_FRAME_REDUCTION: f32 = 0.1; // Every 10 pts = 1 frame less movement. so need 100 to get to 1 frame move. SEEMS OVERPOWERED. it probably is. but want a change in speed for every mutation in this (10). Maybe should just increase the default frames to move from 10 to higher. but that would fuck with everything because you could have things reproducing by the time a deer walks a little lol.
pub static MOVE_SPEED_METABOLISM_MULTIPLIER: f32 = 3. * MOVE_SPEED_FRAME_REDUCTION; // 3 * .1 * 10 = 3 so basically 3x metabolism for every 10 points in this stat. 

pub static BASE_PREGNANCY_TIME_ADDER: i32 = 1; // for every point in this trait, increase pregnancy time.
pub static FAST_GROWER_MULTIPLIER: f32 = 0.005; // each point in fast grower reduces total time by .5%
pub static FAST_GROWER_CALORIE_MULTIPLIER: f32 = 0.005; // each point in fast grower increases total calories by .5%
pub static LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY: i32 = 100;
pub static LITTER_SIZE_METABOLISM_MULTIPLIER: f32 = 1. / LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY as f32;
pub static CANNIBAL_PREGNANCY_CHILD_CALORIES_MULTIPLIER: f32 = 0.1; // so 100 would give u 10x starting calories!
pub static CANNIBAL_PREGNANCY_DEATH_WEIGHT_MULTIPLIER: f32 = 1.0; // so 100 would give u 10x starting calories!

pub static DEFAULT_VISION_RANGE: f32 = 5.;

pub static DEFAULT_SOIL_SPREAD_RATE: u32 = STANDARD_FRAMES_TO_MOVE as u32 * 20;
pub static DEFAULT_BUD_RATE: u32 = STANDARD_FRAMES_TO_MOVE as u32 * 10;

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
    pub evolving_traits: Option<EvolvingTraitsComponent>,
    pub sexual_reproduction: Option<SexualReproduction>,
    pub vision_component: Option<VisionComponent>,
    pub ai_component: Option<AIComponent>,
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
            sexual_reproduction: other.sexual_reproduction.or(self.sexual_reproduction),
            vision_component: other.vision_component.or(self.vision_component),
            ai_component: other.ai_component.or(self.ai_component),
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
            sexual_reproduction: self.sexual_reproduction.clone(),
            vision_component: self.vision_component.clone(),
            ai_component: self.ai_component.clone(),
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
            sexual_reproduction: None,
            vision_component: None,
            ai_component: None,
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
    pub fn at_max_health(&self) -> bool {
        return self.health == self.max_health;
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
pub struct AIComponent {
    pub ai_id: UID,
}
impl Component for AIComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
// Essentially for budding and plant reproduction, don't reproduce onto a tile that already has something with the uses the same soil layer.
// For things that "bud" that don't need any soil remove the soil component, like if I ever make budding animals
// All Type takes up all the soil nothing can grow EXCEPT Free.
pub enum SoilHeight {
    Grass,
    Flower,
    Bush,
    // Tree would just be something with All soil layber and is a blocker basically
    All, // blocks all growth
}
impl Default for SoilHeight {
    fn default() -> Self { SoilHeight::Grass }
}

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
// there are three soil types. Budders have a type that lets them grow on 
// 2 out of 3 soils, and another state that determines which soil type they spread around them.
pub enum SoilType {
    Silt,
    Clay,
    Sand,
}
impl Default for SoilType {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 3) {
            0 => {
                SoilType::Silt
            },
            1 => {
                SoilType::Clay
            },
            2 => {
                SoilType::Sand
            }
            _ => {
                panic!("wtf rng");
            }
        }
    }
}
impl SoilType {
    pub fn map_string(&self) -> String {
        match self {
            SoilType::Silt => "Si".to_string(),
            SoilType::Clay => "Cl".to_string(),
            SoilType::Sand => "Sa".to_string(),
        }
    }
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
impl BuddingComponent {
    pub fn new(reproduction_rate: u32, frame_ready_to_reproduce: u128) -> Self {
        BuddingComponent {
            reproduction_rate: reproduction_rate,
            frame_ready_to_reproduce: frame_ready_to_reproduce,
            seed_creature_differences: Box::new(ComponentMap::fake_default()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct SoilComponent {
    pub soil_height: SoilHeight,
    pub soil_type_cannot_grow: SoilType,
    pub soil_type_spread: SoilType,
    pub frame_ready_to_spread: u128,
    pub spread_rate: Option<u32>, // None means no spreading
}
impl Component for SoilComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Default for SoilComponent {
    fn default() -> Self {
        // 3 different cannot grows and 3 different spreads
        // and 3 different rnged SoilLayers = 27
        let mut rng = rand::thread_rng();

        let soil_layer: SoilHeight = 
        {
            let num = rng.gen_range(0, 10);
            if num <= 2 {
                SoilHeight::Grass
            } else if num <= 5 {
                SoilHeight::Flower
            } else if num <= 8 {
                SoilHeight::Bush
            } else {
                SoilHeight::All // only 1/10 chance for All
            }
        };
        let soil_type_cannot_grow = match rng.gen_range(0, 3) {
            0 => {
                SoilType::Silt
            },
            1 => {
                SoilType::Clay
            },
            2 => {
                SoilType::Sand
            }
            _ => {
                panic!("wtf rng");
            }
        };

        let soil_type_spread = match rng.gen_range(0, 3) {
            0 => {
                SoilType::Silt
            },
            1 => {
                SoilType::Clay
            },
            2 => {
                SoilType::Sand
            }
            _ => {
                panic!("wtf rng");
            }
        };

        SoilComponent {
            soil_height: soil_layer,
            soil_type_cannot_grow,
            soil_type_spread,
            frame_ready_to_spread: DEFAULT_SOIL_SPREAD_RATE as u128,
            spread_rate: Some(DEFAULT_SOIL_SPREAD_RATE),
        }
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


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct VisionComponent {
    pub visible_creatures: Vec<UID>,
    // TODONEXT: need to make a system for this. think it has to be an event thing.
}
impl Component for VisionComponent {
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

#[derive(Debug, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct SexualReproduction {
    pub is_pregnant: bool,
    pub pregnancy_completion_frame: u128,
    pub litter_size: u32,
    pub partner_genes : EvolvingTraits,
}
impl Clone for SexualReproduction {
    fn clone(&self) -> Self {
        Self { is_pregnant: false, pregnancy_completion_frame: 0, litter_size: 0, partner_genes: EvolvingTraits::default() }
    }
}
impl Component for SexualReproduction {
    fn get_visible() -> bool {
        true
    }
}
impl SexualReproduction {
    // TODONEXT make a reproduction system. counts down pregnancy then pops out kids based on this component and relevant evo traits. Also modify metabolsim consumption based on pregnancy and traits
    // TODONEXT make a sex creature command, requires they be same species.
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default)]
#[derive(Deserialize, Serialize)]
pub struct EvolvingTraits {
    //Implemented: 
    pub thick_hide: i32, // Reduces damage taken by a flat amount? increases calories per movement?
    pub sharp_claws: i32, // Increase damage done by attacksimple
    pub girth: i32, // Increase max health, no downside
    
    pub move_speed: i32, // increases running speed but increases metabolism consumed when moving
    pub species: i32, // species should start out fairly far apart from each other (this number is random). larger the diff in this number the lower chance they can reproduce sucessfully during sex. 
    pub litter_size: i32, // increases children but also increases metabolism while pregnant. just bad if this is negative... maybe increases chance of no pregnancy/miscarriage?
    pub pregnancy_time: i32, // increases pregnancy time but children come out more developed, and vice versa
    pub maleness: i32, // all animals are hermaphrodite, maleness makes it so u have less chance of being the one who becomes pregnant when sex. Need to also implement ways for animals to detect males in their AI and if they are also male themselves, this might cause male-male competition to evolve hopefully?
    pub fast_grower: i32, // decreases time as child once out of womb, but increases metabolism
    pub cannibal_childbirth: i32, // chance to die in childbirth, but babies born with a lot of calories

    //Unimplemented:
    //move_speed, make it increase metabolism while doing

    pub hamstring: i32, // lowers speed of victim after attacksimple? need to make a whole status effect component for this? or add to movement componenet (prob that?)? how to do duration though and magnitude? debug stack shud prob make a whoel component then for status conditions?
    pub more_mutations: i32, // Higher the number more mutations that will happen, similar to litter_size in terms of probability and guarantees after 100
    // pub graceful: i32, // higher this value the less ur metabolism consumed when moving. BAD because move_speed is OP as fuck and its cost is food so fuck buffing it by making this exist.
    // pub anti_rape: i32, // increases chance of sex failing when other creatures try to sex you.
}
impl EvolvingTraits {
    pub fn clone_with_multiplier_and_exceptions(&self, multiplier :f32, ignore_child_exceptions: bool) -> EvolvingTraits {
        EvolvingTraits {
            thick_hide: (self.thick_hide as f32 * multiplier) as i32,
            sharp_claws: (self.sharp_claws as f32 * multiplier) as i32,
            hamstring: (self.hamstring as f32 * multiplier) as i32,
            move_speed: (self.move_speed as f32 * multiplier) as i32,
            species: (self.species as f32 * multiplier) as i32,
            litter_size: (self.litter_size as f32 * multiplier) as i32,
            pregnancy_time: (self.pregnancy_time as f32 * multiplier) as i32,
            maleness: (self.maleness as f32 * multiplier) as i32,
            fast_grower: if ignore_child_exceptions {self.fast_grower } else { (self.fast_grower as f32 * multiplier) as i32}, // childness should not affect fast_grower as it affects childness which is weird.
            girth: (self.girth as f32 * multiplier) as i32,
            more_mutations: (self.more_mutations as f32 * multiplier) as i32,
            cannibal_childbirth: (self.cannibal_childbirth as f32 * multiplier) as i32,
        }
    }

    pub fn mix_traits(a: i32, b: i32) -> i32{
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 3) {
            0 => {
                a
            },
            1 => {
                b
            },
            2 => {
                (a + b) / 2
            }
            _ => {
                panic!("wtf rng");
            }
        }
    }

    pub fn clone_with_mate(&self, mate: &EvolvingTraits) -> EvolvingTraits {
        EvolvingTraits {
            thick_hide: EvolvingTraits::mix_traits(self.thick_hide, mate.thick_hide),
            sharp_claws: EvolvingTraits::mix_traits(self.sharp_claws, mate.sharp_claws),
            hamstring: EvolvingTraits::mix_traits(self.hamstring, mate.hamstring),
            move_speed: EvolvingTraits::mix_traits(self.move_speed, mate.move_speed),
            species: EvolvingTraits::mix_traits(self.species, mate.species),
            litter_size: EvolvingTraits::mix_traits(self.litter_size, mate.litter_size),
            pregnancy_time: EvolvingTraits::mix_traits(self.pregnancy_time, mate.pregnancy_time),
            maleness: EvolvingTraits::mix_traits(self.maleness, mate.maleness),
            fast_grower: EvolvingTraits::mix_traits(self.fast_grower, mate.fast_grower),
            girth: EvolvingTraits::mix_traits(self.girth, mate.girth),
            more_mutations: EvolvingTraits::mix_traits(self.more_mutations, mate.more_mutations),
            cannibal_childbirth: EvolvingTraits::mix_traits(self.cannibal_childbirth, mate.cannibal_childbirth),
        }
    }
}

// to allow AI to eventually get smart enough to figure out that they should
// chop down trees to make navigation easier gonna add a "breakable" bool here.
// ACTUALLY I wont cause that should be based on like health component existing
// or something like that
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default)]
#[derive(Deserialize, Serialize)]
pub struct EvolvingTraitsComponent {
    pub adult_traits: EvolvingTraits,
    pub traits: EvolvingTraits,
    
    pub child_until_frame: u128, // used to calculate childness
    pub born_on_frame: u128, // used for childness
}
impl Component for EvolvingTraitsComponent {
    fn get_visible() -> bool {
        true
    }
}
impl EvolvingTraitsComponent {
    pub fn get_if_child(&self, frame: u128) -> bool {
        return frame < self.child_until_frame;
    }

    pub fn get_litter_size(&self) -> u32 {
        let mut total = 1;
        let mut rng = rand::thread_rng();
        if self.traits.litter_size < 0 {
            let abort_chance = self.traits.litter_size as f64 / LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY as f64;
            if rng.gen_bool(abort_chance.abs()) {
                return 0;
            } else {
                return 1;
            }
        }

        let guaranteed_added = self.traits.litter_size / LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY;
        total += guaranteed_added;
        let increase_chance = (self.traits.litter_size % LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY) as f64 / LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY as f64;
        if rng.gen_bool(increase_chance) {
            total += 1;
        }

        total as u32
    }

    pub fn get_adult_percent(&self, frame: u128) -> f32 {
        if frame >= self.child_until_frame {
            return 1.;
        }
        let total_time = self.child_until_frame - self.born_on_frame;
        let percent_done = (frame - self.born_on_frame) as f32 / total_time as f32;
        return percent_done;
    }

    pub fn update_stats_based_on_childness(&mut self, frame: u128 ) {
        if frame > self.child_until_frame {
            // if below assertion fails make sure child_until_frame was set to at least 1 to give a frame to update
            assert!(self.adult_traits == self.traits);
            return;
        }
        let total_time = self.child_until_frame - self.born_on_frame;
        let mut percent_done = (frame - self.born_on_frame) as f32 / total_time as f32;
        if total_time == 0 {
            percent_done = 1.;
        }
        self.traits = self.adult_traits.clone_with_multiplier_and_exceptions(percent_done, true);
        // TODO whatever calls this should remove child component when this is done?
        // TODO: Fuck this is gonna be really complicated for stuff like healthcomponent, movement etc that set their own stats at startup.
        // For example, max hp is set when creature is born, so will have to fucking update max hp every frame as child grows.
        // gonna be tedious for every system... maybe make on "on init" type function that does this? also used when a child is born
    }

    pub fn get_newborn_starting_calories_multiplier(&self) -> f32 {
        // NOTE no downside to getting more than 100 cannibal_childbirth so maybe worth clamping to 100? or maybe its ok
        return 1.0 + (self.traits.cannibal_childbirth as f32 * CANNIBAL_PREGNANCY_CHILD_CALORIES_MULTIPLIER);
    }

    pub fn get_weight_of_childbirth_death(&self) -> i32 {
        (self.traits.cannibal_childbirth as f32 * CANNIBAL_PREGNANCY_DEATH_WEIGHT_MULTIPLIER) as i32
    }

    pub fn get_frames_to_move(&self) -> usize {
        return std::cmp::max(STANDARD_FRAMES_TO_MOVE - (MOVE_SPEED_FRAME_REDUCTION * self.traits.move_speed as f32) as i32, 1) as usize;
    }

    pub fn get_max_health(&self) -> i32 {
        return (STANDARD_HP as f32 + (self.traits.girth as f32 * GIRTH_HEALTH_INCREASE)) as i32;
    }

    pub fn get_pregnancy_weight(&self) -> i32 {
        return std::cmp::max(BASE_PREGNANCY_CHANCE_WEIGHT - self.traits.maleness, 0);
    }

    pub fn get_pregnancy_length(&self) -> u128 {
        return std::cmp::max(STANDARD_PREGNANCY_TIME as i32 + (self.traits.pregnancy_time * BASE_PREGNANCY_TIME_ADDER) as i32, 1) as u128;
    }

    pub fn get_vision_range(&self) -> f32 {
        // TODO: make a trait for vision range
        // maybe encorporate different senses like smell and stuff eventually
        DEFAULT_VISION_RANGE
    }

    pub fn get_mutated(&self, mutations: u32) -> EvolvingTraitsComponent {
        let mut child = self.clone();

        let mut rng = rand::thread_rng();
        (0..mutations).for_each(|_| {
            let change: i32 = if rng.gen_bool(0.5) {
                MUTATION_CHANGE
            } else {
                -1 * MUTATION_CHANGE
            };
            let chosen = rng.gen_range(0, 10);
            match chosen {
                0 => {
                    child.adult_traits.thick_hide += change;
                },
                1 => {
                    child.adult_traits.sharp_claws += change;
                },
                2 => {
                    child.adult_traits.girth += change;
                },
                3 => {
                    child.adult_traits.move_speed += change;
                },
                4 => {
                    child.adult_traits.species += change;
                },
                5 => {
                    child.adult_traits.litter_size += change;
                },
                6 => {
                    child.adult_traits.pregnancy_time += change;
                },
                7 => {
                    child.adult_traits.maleness += change;
                },
                8 => {
                    child.adult_traits.fast_grower += change;
                },
                9 => {
                    child.adult_traits.cannibal_childbirth += change;
                },
                _ => {
                    panic!("Got to an unimplemented mutation");
                },
            }
        });
        child
    }


    // the below functions used in the simple_attack creature command code
    pub fn get_total_simple_attack_adder(&self) -> i32 {
        let mut total = SIMPLE_ATTACK_BASE_DMG;
        total += (self.traits.sharp_claws as f32 * SHARP_CLAWS_DMG_INCREASE).floor() as i32;
        total
    }

    pub fn get_total_defense_subtractor(&self) -> i32 {
        let mut total = 0;
        total += (self.traits.thick_hide as f32 * THICK_HIDE_DMG_REDUCE_MULTIPLIER).floor() as i32;
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
            moving: false, // TODO: hopefully this is enough for children to not bug out?
            frame_ready_to_move: self.frame_ready_to_move,
        }
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct StarvationComponent {
    pub calories: i32,
    pub metabolism: usize, // is the base calories spent per frame
}
impl Component for StarvationComponent {
    fn get_visible() -> bool {
        true
    }
}
impl Clone for StarvationComponent {
    fn clone(&self) -> Self {
        StarvationComponent{
            calories: STANDARD_METABOLISM as i32 * REPRODUCE_STARTING_CALORIES_MULTIPLIER,
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
#[derive(Hash, PartialEq, Eq)]
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
impl Clone for BattleComponent {
    fn clone(&self) -> Self {
        Self { in_battle: None }
    }
}
impl Component for BattleComponent {
    fn get_visible() -> bool {
        true
    }
}
