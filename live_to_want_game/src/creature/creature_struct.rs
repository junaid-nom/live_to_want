use std::fmt::Formatter;

use crate::{map_state::Item, utils::Vector2};

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
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct CreatureState {
    pub components: ComponentMap,
    pub memory: CreatureMemory,
    pub inventory: Vec<Item>,
}
impl CreatureState {
    pub fn new<'a>(loc: Vector2) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = LocationComponent{location:loc};
        ret
    }

    // for reproduction via budding mostly
    pub fn copy(c: &CreatureState, new_loc: Vector2) -> CreatureState {
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
            starvation_component: if let Some(s) = c.components.starvation_component.as_ref() {
                Some(StarvationComponent{
                    calories: s.metabolism as i32 * REPRODUCE_STARTING_CALORIES,
                    metabolism: s.metabolism,
                })
            } else {
                None
            },
            creature_type_component: c.components.creature_type_component.clone(),
            block_space_component: c.components.block_space_component.clone(),
            movement_component: c.components.movement_component.clone(),
        };

        CreatureState {
            components: cmap,
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        }
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
        let mut f_string = String::new();
        for item in &self.inventory {
            f_string = format!("{},{}",f_string, item.quantity);
        }
        write!(f, "{}", f_string)
    }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq)]
pub struct CreatureMemory {
    
}

