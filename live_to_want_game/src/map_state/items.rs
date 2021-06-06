use crate::{Battle, Location, creature::CreatureState, tasks::Event, tasks::EventChain, tasks::EventTarget, tasks::EventType, utils::UID, utils::Vu2};

use super::MapLocation;

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
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


#[derive(Debug)]
pub enum CreatureCommand<'b>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'static str, &'b CreatureState, Location, u128), // Assume this sets the destination not instantly move to
    Chase(&'static str, &'b CreatureState, &'b CreatureState),
    Attack(&'static str, &'b CreatureState, &'b CreatureState, UID),
    TakeItem(&'static str, InventoryHolder<'b>, InventoryHolder<'b>, Item),
}
impl CreatureCommand<'_> {
    pub fn to_event_chain(&self) -> Option<EventChain> {
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
                    events: vec![init_move]
                });
            }
            CreatureCommand::Chase(_, _, _) => {}
            CreatureCommand::Attack(_, attacker, victim, battle_list_id) => {
                // Create two events that set in battle and battle started = false for the creatures.
                // And a AddBattle event that will add a battle to a list of battles on mapState.
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
                    target: attacker.components.id_component.id()
                };
                let remove_p1_when_fail = Event {
                    event_type: EventType::LeaveBattle(),
                    get_requirements: Box::new(|_,_| true),
                    on_fail: None,
                    target: attacker.components.id_component.id()
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
                        events: vec![remove_p1_when_fail]
                    }),
                    target: victim.components.id_component.id()
                };
                let start_battle = Event {
                    event_type: EventType::AddBattle(battle),
                    get_requirements: Box::new(|_,_| true),
                    on_fail: None,
                    target: *battle_list_id,
                };
                
                return Some(EventChain {
                    events: vec![add_p1, add_p2, start_battle]
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
