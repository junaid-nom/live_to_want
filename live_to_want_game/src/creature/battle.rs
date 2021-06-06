use rand::Rng;

use crate::{CreatureState, Location, Vu2, map_state::MapState, tasks::Event, tasks::EventChain, tasks::EventType};

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
pub enum Attacks {
    SimpleDamage(i32)
}
impl Default for Attacks {
    fn default() -> Self { Attacks::SimpleDamage(1) }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct BattleInfo {
    pub health: i32,
    pub max_health: i32,
    pub attacks: Vec<Attacks>,
    pub creature_id: u64,
    pub creature_location: Location,
}
impl BattleInfo {
    pub fn new(c: &CreatureState) -> Self {
        let health_c = c.components.health_component.as_ref().unwrap();
        let id_c = c.components.id_component.id();
        let loc = c.get_location();
        // TODO: actually make interesting creatures and attacks
        BattleInfo {
            health: health_c.health,
            max_health: health_c.max_health,
            attacks: vec![Attacks::default()],
            creature_id: id_c,
            creature_location: loc,
        }
    }
}

pub struct Battle {
    pub fighter1: BattleInfo,
    pub fighter2: BattleInfo,
    pub frame_started: u128
}
impl Battle {
    pub fn new(c1: &CreatureState, c2: &CreatureState, current_frame: u128) -> Self {
        Battle {
            fighter1: BattleInfo::new(c1),
            fighter2: BattleInfo::new(c2),
            frame_started: current_frame,
        }
    }
}

