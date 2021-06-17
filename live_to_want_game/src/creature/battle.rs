use std::fmt::Display;

use rand::{Rng, prelude::SliceRandom};

use crate::{BattleFrame, CreatureState, Item, Location, UID, Vu2, get_id, map_state::MapState, tasks::Event, tasks::EventChain, tasks::EventType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
pub enum Attack {
    SimpleDamage(BattleFrame, i32), // frames to execute, damage should be positive val
    DoNothing()
}
impl Attack {
    pub fn get_attack_frame_speed(&self) -> BattleFrame {
        match &self {
            Attack::SimpleDamage(frames, _) => *frames,
            Attack::DoNothing() => 0,
        }
    }
}
impl Default for Attack {
    fn default() -> Self { Attack::SimpleDamage(3, 1) }
}
impl Display for Attack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let f_string = match self {
            Attack::SimpleDamage(delay, dmg) => format!("Simple att dmg:{},t:{}", dmg, delay),
            Attack::DoNothing() => "Nothing".to_string(),
        };
        write!(f, "{}", f_string)
    }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
pub struct BattleInfo {
    pub health: i32,
    pub max_health: i32,
    pub attacks: Vec<Attack>,
    pub creature_id: UID,
    pub creature_location: Location,
    pub last_attack_frame: BattleFrame,
    pub current_attack: Attack,
    pub items: Vec<Item>,
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
            attacks: vec![Attack::default()],
            creature_id: id_c,
            creature_location: loc,
            last_attack_frame: 0,
            current_attack: Attack::DoNothing(),
            items: c.inventory.clone()
        }
    }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone)]
#[derive(Deserialize, Serialize)]
// TODO KINDA: Make this an interface so you can swap out different battle systems entirely lol?
pub struct Battle {
    pub fighter1: BattleInfo,
    pub fighter2: BattleInfo,
    // Battle uses its own frame. This way you can for example, don't increase frame until human player sends command. or increase frame after X seconds. or run throuh an entire battle in a single map state frame. detached from map state frame change.
    pub frame: BattleFrame, 
    pub id: UID,
    pub battle_list_id: UID,
}
impl Battle {
    pub fn new(c1: &CreatureState, c2: &CreatureState, battle_list_id: UID) -> Self {
        Battle {
            fighter1: BattleInfo::new(c1),
            fighter2: BattleInfo::new(c2),
            frame: 0,
            id: get_id(),
            battle_list_id
        }
    }
    pub fn update(&mut self) -> Option<EventChain> {
        self.frame += 1;
        let frame = self.frame;
        let battle_id = self.id;
        let mut fighters = vec![&mut self.fighter1, &mut self.fighter2];
        // go through battle infos, check if ready to cast. return attack tuple (attack, target_index)
        let attack_tuples: Vec<Option<(Attack, usize, usize)>> = fighters.iter().enumerate().map(
            |(index, fighter)| {
                if fighter.current_attack.get_attack_frame_speed() + fighter.last_attack_frame <= frame {
                    let attack = fighter.current_attack;
                    let my_index = index;
                    let enemy_index = (index + 1) % 2;
                    Some((attack, my_index, enemy_index))
                } else {
                    None
                }
            }
        ).collect();

        let mut rng = rand::thread_rng();
        attack_tuples.into_iter().for_each(|tuple| {
            match tuple {
                Some((attack, attacker, victim )) => {
                    match attack {
                        Attack::SimpleDamage(_, dmg) => fighters[victim].health -= dmg,
                        Attack::DoNothing() => {},
                    }
                    // set next attack as well. for now just pick random attack in attacks
                    // TODONEXT base it on some async stuff.
                    fighters[attacker].last_attack_frame = frame;
                    fighters[attacker].current_attack = *fighters[attacker].attacks.choose(&mut rng).unwrap();
                },
                None => {},
            }
        });

        // check if either fighter dead, then create EventChain for results
        let victor = if fighters[0].health <= 0 && fighters[1].health <= 0 {
            2 // both lose
        } else if fighters[0].health <= 0 {
            1
        } else if fighters[1].health <= 0 {
            0
        } else {
            -1
        };

        // event chain should move items then set HPs 
        // also should remove from combat, and remove battle from battle list
        if victor >= 0 {
            println!("Battle {} finished! victor: {}", self.id, victor);
            let mut end_combat = vec![
                Event::make_basic(EventType::LeaveBattle(), fighters[0].creature_id),
                Event::make_basic(EventType::LeaveBattle(), fighters[1].creature_id),
                Event::make_basic(EventType::RemoveBattle(self.id), self.battle_list_id)
            ];

            let mut single_winner = |winner: usize, loser: usize| {
                let set_winner_hp = Event::make_basic(EventType::SetHealth(fighters[winner].health), fighters[winner].creature_id);
                let set_loser_hp = Event::make_basic(EventType::SetHealth(fighters[loser].health), fighters[loser].creature_id);
                let mut move_items: Vec<Event> = fighters[loser].items.iter().flat_map(|item| {
                    let mut events = vec![];
                    events.push(Event::make_basic(EventType::RemoveItem(item.quantity, item.item_type), fighters[loser].creature_id));
                    events.push(Event::make_basic(EventType::AddItem(item.quantity, item.item_type), fighters[winner].creature_id));
                    events
                }).collect();
                move_items.push(set_winner_hp);
                move_items.push(set_loser_hp);
                move_items.append(&mut end_combat);
                let ec = EventChain {
                    events: move_items,
                    debug_string: format!("Battle {} Finished", battle_id),
                    creature_list_targets: false,
                };
                println!("Battle events: {}", ec.events.len());
                ec
            };

            match victor {
                0 => {
                    // fighter index 0 won
                    Some(single_winner(0, 1))
                },
                1 => {
                    // fighter index 1 won
                    Some(single_winner(1, 0))
                },
                2 => {
                    // both dead
                    end_combat.push(Event::make_basic(EventType::SetHealth(fighters[0].health), fighters[0].creature_id));
                    end_combat.push(Event::make_basic(EventType::SetHealth(fighters[1].health), fighters[1].creature_id));
                    Some(EventChain {
                        events: end_combat,
                        debug_string: format!("Battle {} Finished both dead", battle_id),
                        creature_list_targets: false,
                    })
                },
                _ => panic!("match with invalid victor?")
            }
        } else {
            None
        }

        //TODONEXT: Need to actually make this async. So basically, have some async thread that runs AI/gets human input.
        // then those commands get fed into battles.
        // probably make map_state only update once every X frames per second. if player is in a battle maybe at a slower speed?
        // so basically actually make async user input work...
        // should feel like MMO combat basically. kinda turn based but with real time.
    }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BattleList {
    pub battles: Vec<Battle>,
    pub id: UID
}
impl BattleList {
    pub fn new() -> Self {
        BattleList {
            battles: vec![],
            id: get_id()
        }
    }
}
