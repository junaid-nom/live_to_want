use std::io::stderr;

use crate::{Battle, Location, MAX_ATTACK_DISTANCE, MapState, creature::CreatureState, tasks::Event, tasks::EventChain, tasks::EventTarget, tasks::EventType, utils::UID, utils::Vu2};
use serde::{Deserialize, Serialize};

use super::MapLocation;

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
#[derive(Deserialize, Serialize)]
pub enum ItemType {
    Berry,
    Meat,
    Bones,
    Wood,
}
impl Default for ItemType {
    fn default() -> Self { ItemType::Berry }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone, Copy)]
#[derive(Deserialize, Serialize)]
pub struct Item {
    pub item_type: ItemType,
    pub quantity: u32,
}
impl Item {
    pub fn new(item_type: ItemType, quantity:u32) -> Self {
        Item {
            item_type, quantity
        }
    }
}

/// Must be Copy/Clone easily.
#[derive(Debug, Copy, Clone)]
#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum CreatureCommandUser {
    // TODONEXT: Make stuff that users can send here, get turned into CreatureCommands if they meet requirements
    MoveTo(UID, Location), // creature, target loc
    Attack(UID, UID) // attacker, victim
}
impl CreatureCommandUser {
    pub fn to_creature_command<'a, 'b>(&self, map_state :&'b MapState, c_state : &'b CreatureState) -> Option<CreatureCommand<'b>> {
        // match on self, check if the command is even legal. For example, moving to impossible location. 
        // also that the user owns the creature (or should that be done earlier)?
        // then turn the command into a creature command by getting refs from mapstate.
        // then in main game loop par_iter on all messages with CreatureCommandUser to generate these.
        // then generate the event chains and perform them.

        // Also need to add an if for regular goal generation, to not do it if its a user-owned creature and its not set to "auto"
        todo!()
    }
}


#[derive(Debug)]
pub enum CreatureCommand<'b>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'static str, &'b CreatureState, Location, u128), // Assume this sets the destination not instantly move to
    Chase(&'static str, &'b CreatureState, &'b CreatureState),
    Attack(&'static str, &'b CreatureState, &'b CreatureState, UID), // attacker, victim, 3rd is battle list uid
    TakeItem(&'static str, InventoryHolder<'b>, InventoryHolder<'b>, Item),
}
impl CreatureCommand<'_> {
    pub fn to_event_chain(&self) -> Option<EventChain> {
        // TODO: Need to at some point verify all creature commands are valid probably earlier than here, somewhere when we get input from AI/human players
        // Especially stuff like "take item"... probably need to have some kind of like "Admin commands" and "Player commands" idk? Because takeItem used by engine probably?
        // actually take item ISNT used by anything hmm also does that mean its fucking untested lol?
        // stuff like move_to probably still needs limits at least?
        match self {
            CreatureCommand::MoveTo(_, c, destination, current_frame) => {
                // initialize movement component to new destination
                let init_move = Event {
                    event_type: EventType::InitializeMovement(*current_frame, *destination),
                    on_fail: None,
                    get_requirements: Box::new(|_,_| true),
                    target: c.components.id_component.id(),
                };
                return Some(EventChain{
                    events: vec![init_move],
                    debug_string: format!("Move to {:?} for {}", destination, c.components.id_component.id()),
                    creature_list_targets: true,
                });
            }
            CreatureCommand::Chase(_, _, _) => {}
            CreatureCommand::Attack(_, attacker, victim, battle_list_id) => {
                // Create two events that set in battle and battle started = false for the creatures.
                // And a AddBattle event that will add a battle to a list of battles on mapState.

                let dist = attacker.get_location().distance_in_region(&victim.get_location());
                match dist {
                    Some(dist) => {
                        if dist > MAX_ATTACK_DISTANCE {
                            println!("Trying to attack enemy out of range!");
                            return None
                        }
                    },
                    None => {
                        println!("Trying to attack enemy not even in same region!");
                        return None
                    },
                }

                // NOTE: MUST order the set in combat events by lowest ID! That way if two enemies fight each other same time
                // it'll work. otherwise u get deadlock
                let attacker_is_lower = attacker.get_id() < victim.get_id();
                let lower_id = if attacker_is_lower {
                    attacker
                } else {
                    victim
                };
                let higher_id = if attacker_is_lower {
                    victim
                } else {
                    attacker
                };
                let battle = Battle::new(attacker,victim, *battle_list_id);
                let add_p1 = Event {
                    event_type: EventType::EnterBattle(battle.id),
                    get_requirements: Box::new(|e, _| {
                        match e {
                            EventTarget::CreatureTarget(c) => {
                                return !c.get_if_in_combat()
                            }
                            _ => {
                                panic!("Got eventtarget that isnt for items")
                            }
                        }
                    }),
                    on_fail: None,
                    target: lower_id.components.id_component.id()
                };
                
                let remove_p1_when_fail = Event {
                    event_type: EventType::LeaveBattle(),
                    get_requirements: Box::new(|_,_| true),
                    on_fail: None,
                    target: lower_id.components.id_component.id()
                };

                let add_p2 = Event {
                    event_type: EventType::EnterBattle(battle.id),
                    get_requirements: Box::new(|e, _| {
                        match e {
                            EventTarget::CreatureTarget(c) => {
                                return !c.get_if_in_combat()
                            }
                            _ => {
                                panic!("Got eventtarget that isnt for items")
                            }
                        }
                    }),
                    on_fail: Some(EventChain{
                        events: vec![remove_p1_when_fail],
                        debug_string: format!("Fail attack from {} to {}", attacker.components.id_component.id(), victim.components.id_component.id()),
                        creature_list_targets: false,
                    }),
                    target: higher_id.components.id_component.id()
                };
                let start_battle = Event {
                    event_type: EventType::AddBattle(battle),
                    get_requirements: Box::new(|_,_| true),
                    on_fail: None,
                    target: *battle_list_id,
                };
                return Some(EventChain {
                    events: vec![add_p1, add_p2, start_battle],
                    debug_string: format!("attack from {} to {}", attacker.components.id_component.id(), victim.components.id_component.id()),
                    creature_list_targets: false,
                });
            }
            CreatureCommand::TakeItem(_, src, dst, item) => {
                // TODO: check if dst has enough space, though maybe just have "cant move" if your inv full
                // check if src has that item, if it doesnt, take as many as possible
                let found_item = get_item_from_inventory(src, item.item_type);
                if let None = found_item {
                    return None;
                }
                let found_item = found_item.unwrap();
                let final_item = if found_item.quantity < item.quantity {
                    found_item
                } else {
                    *item
                };

                // event chain is:
                // remove item from src. req=item exists in that quantity fail=None
                // add item to dst. req=None(for now) fail=None
                let remove = Event{
                    event_type: EventType::RemoveItem(final_item.quantity, item.item_type),
                    target: get_id_from_inventory(src),
                    on_fail: None,
                    get_requirements: Box::new(|e, et| {
                        if let EventType::RemoveItem(q, it) = et {
                            match e {
                                EventTarget::LocationItemTarget(i, _) => {
                                    for item in i.iter() {
                                        if item.item_type == *it && item.quantity >= *q {
                                            return true
                                        }
                                    }
                                    return false
                                }
                                EventTarget::CreatureTarget(c) => {
                                    for item in c.inventory.iter() {
                                        if item.item_type == *it && item.quantity >= *q {
                                            return true
                                        }
                                    }
                                    return false
                                }
                                _ => {
                                    panic!("Got eventtarget that isnt for items")
                                }
                            }
                        }
                        false
                    })
                };
                let add = Event {
                    event_type: EventType::AddItem(final_item.quantity, item.item_type),
                    on_fail: None,
                    get_requirements: Box::new(|_,_| true),
                    target: get_id_from_inventory(dst),
                };
                return Some(EventChain{
                    events: vec![remove, add],
                    debug_string: format!("Take item {:?} for {}", final_item, get_id_from_inventory(dst)),
                    creature_list_targets: false,
                })
            }
        }
        None
    }
}

fn get_id_from_inventory(inv: &InventoryHolder) -> UID {
    match inv {
        InventoryHolder::CreatureInventory(c) => {c.components.id_component.id()}
        InventoryHolder::LocationInventory(l) => {l.id_component_items.id()}
    }
}

fn get_item_from_inventory(inv: &InventoryHolder, item_type: ItemType) -> Option<Item> {
    match inv {
        InventoryHolder::CreatureInventory(c) => {
            get_item_from_vec_item(&c.inventory,item_type)
        }
        InventoryHolder::LocationInventory(l) => {
            get_item_from_vec_item(&l.items,item_type)
        }
    }
}
fn get_item_from_vec_item(vec_inv: &Vec<Item>, item_type: ItemType) -> Option<Item> {
    for i in vec_inv {
        if i.item_type == item_type {
            return Some(*i)
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub enum InventoryHolder<'a> {
    CreatureInventory(&'a CreatureState),
    LocationInventory(&'a MapLocation),
}
