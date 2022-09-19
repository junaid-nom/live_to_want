use std::{fmt::Formatter, cmp::max};
use serde::{Deserialize, Serialize};
use crate::{Location, RegionComponent, UID, map_state::Item, utils::Vector2, utils::Vu2, UserComponent, STANDARD_PREGNANCY_TIME, STANDARD_CHILD_TIME, FAST_GROWER_MULTIPLIER, SPECIES_SEX_RANGE};

use super::{ComponentMap, IDComponent, LocationComponent, HealthComponent, NameComponent, StarvationComponent, REPRODUCE_STARTING_CALORIES};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum CreatureType {
    Deer,
    Wolf,
    Human,
    Tree,
}
impl Default for CreatureType {
    fn default() -> Self { CreatureType::Deer }
}

// Components have a func "get_is_visible()"
#[derive(Debug, Clone)]
#[derive(Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct CreatureState {
    pub components: ComponentMap,
    pub memory: CreatureMemory,
    pub inventory: Vec<Item>,
}
impl CreatureState {
    pub fn new<'a>(loc: Vu2) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = LocationComponent{location:loc};
        ret.components.region_component = RegionComponent{region:Vu2::new(0,0)};
        ret
    }

    pub fn new_user_creature<'a>(loc: Location, username: String) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = LocationComponent{location:loc.position};
        ret.components.region_component = RegionComponent{region:loc.region};
        ret.components.user_component = Some(UserComponent{username});
        ret
    }

    pub fn new_location<'a>(loc: Location) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = LocationComponent{location:loc.position};
        ret.components.region_component = RegionComponent{region:loc.region};
        ret
    }

    pub fn can_sex_anything(&self, frame: u128) -> bool {
        if self.components.sexual_reproduction.is_none() || self.components.evolving_traits.is_none() {
            return false;
        }
        // cant if pregnant, and not child
        if self.components.sexual_reproduction.as_ref().unwrap().is_pregnant {
            return false;
        }
        if self.components.evolving_traits.as_ref().unwrap().get_if_child(frame) {
            return false;
        }
        true
    }

    pub fn can_sex(&self, other_species: i32, frame: u128) -> bool {
        if !self.can_sex_anything(frame) {
            return false;
        }
        // cant if pregnant, and not child and not same species
        if (self.components.evolving_traits.as_ref().unwrap().adult_traits.species - other_species).abs() > SPECIES_SEX_RANGE {
            return false;
        }
        
        true
    }

    pub fn get_child_length(&self, mother_pregnany_time: u128) -> u128 {
        let mut total_time = STANDARD_CHILD_TIME.saturating_sub(mother_pregnany_time - STANDARD_PREGNANCY_TIME);

        total_time = (total_time as f32 * (1.0 - (FAST_GROWER_MULTIPLIER * self.components.evolving_traits.as_ref().unwrap().adult_traits.fast_grower as f32)).max(0.0)) as u128;

        return total_time;
    }

    pub fn setup_creature(&mut self, frame: u128, reset_health: bool) {
        // setup childness
        self.components.evolving_traits.as_mut().unwrap().update_stats_based_on_childness(frame);

        // Setup max health and health.
        let max_health = self.components.evolving_traits.as_ref().unwrap().get_max_health();
        let already_max_health = self.components.health_component.as_ref().unwrap().at_max_health();
        self.components.health_component.as_mut().unwrap().max_health = max_health;
        if reset_health || already_max_health {
            self.components.health_component.as_mut().unwrap().health = max_health;
        }
        // setup movement stuff
        self.components.movement_component.as_mut().unwrap().frames_to_move = self.components.evolving_traits.as_ref().unwrap().get_frames_to_move();
    }

    // for reproduction via budding mostly
    pub fn clone_to_new_location(c: &CreatureState, new_loc: Vu2) -> CreatureState {
        // TODO: make all components implement copy/clone traits so its easy to copy em
        // then use default for inventory and memory
        let cmap = ComponentMap{
            id_component: IDComponent::new(),
            health_component: if let Some(h) = c.components.health_component.as_ref() {
                Some(HealthComponent{
                    max_health: h.max_health,
                    health: h.max_health,
                })
            } else {
                None
            },
            region_component: c.components.region_component.clone(),
            location_component: LocationComponent {location: new_loc},
            name_component: Some(NameComponent {}),
            starvation_component: c.components.starvation_component.clone(),
            creature_type_component: c.components.creature_type_component.clone(),
            block_space_component: c.components.block_space_component.clone(),
            movement_component: c.components.movement_component.clone(),
            budding_component: c.components.budding_component.clone(),
            death_items_component: c.components.death_items_component.clone(),
            battle_component:  c.components.battle_component.clone(),
            soil_component: c.components.soil_component.clone(),
            user_component: c.components.user_component.clone(),
            evolving_traits: c.components.evolving_traits.clone(),
            sexual_reproduction: c.components.sexual_reproduction.clone(),
        };

        CreatureState {
            components: cmap,
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        }
    }

    pub fn get_location(&self) -> Location {
        Location::new(self.components.region_component.region, self.components.location_component.location)
    }

    pub fn get_if_in_combat(&self) -> bool {
        match &self.components.battle_component {
            Some(b) => {b.in_battle.is_some()}
            None => {false}
        }
    }

    pub fn get_id(&self) -> UID {
        self.components.id_component.id()
    }
}
impl Default for CreatureState {
    fn default() -> Self {
        CreatureState{
            components: ComponentMap::default(),
            memory: CreatureMemory::default(),
            inventory: Vec::new(),
        }
    }
}
impl std::fmt::Display for CreatureState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut f_string = format!("ID:{} ", self.components.id_component.id());
        if let Some(hc) = self.components.health_component.as_ref() {
            f_string = format!("{} | hp: {}/{}", f_string, hc.health, hc.max_health);
        } else {
            f_string = format!("{} | hp -/-", f_string);
        }

        if let Some(bc) = self.components.battle_component.as_ref() {
            f_string = format!("{} | Combat: {}", f_string, bc.in_battle.as_ref().unwrap_or(&0));
        } else {
            f_string = format!("{} | Combat: N/A", f_string);
        }

        f_string = format!("{} | items ", f_string);
        for item in &self.inventory {
            f_string = format!("{}, {:?}-{}",f_string, item.item_type, item.quantity);
        }
        write!(f, "{}", f_string)
    }
}

#[derive(Debug, Clone)]
#[derive(Default, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct CreatureMemory {
    
}

