use fmt::Debug;

use crate::{
    creature::CreatureState, map_state::Item, map_state::ItemType, map_state::MapState, utils::UID,
    CreatureList, Location,
};
use core::fmt;
use std::collections::HashMap;
extern crate rayon;
use rayon::prelude::*;

/// Is a list of all events for that target for a given frame cycle
/// Must place all tasks for that target in here at once or could cause race conditions
//#[derive(std::marker::Sized)] doesnt work...
pub struct TaskList<'a, 'b> {
    pub target: &'a mut EventTarget<'b>,
    pub tasks: Vec<EventChain>,
}
impl TaskList<'_, '_> {
    pub fn process(mut self) -> Vec<Option<EventChain>> {
        let mut ret = Vec::new();
        for task in self.tasks.into_iter() {
            ret.push(task.process(&mut self.target));
        }
        ret
    }
}

#[derive(Debug)]
pub struct EventChain {
    //pub index: usize,
    pub events: Vec<Event>,
}
impl EventChain {
    fn process(mut self, effected: &mut EventTarget) -> Option<EventChain> {
        let e = self.events.remove(0);
        let success = (*e.get_requirements)(&*effected, &e.event_type);
        if success {
            let added_event = e.mutate(effected);
            let mut se = self;
            if let Some(e) = added_event {
                se.events.push(e);
            }
            if se.events.len() > 0 {
                Some(se)
            } else {
                None
            }
        } else {
            e.on_fail // TODONEXT: Should it return e.onfail? I should get multi_thread tests working to find out!
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventTarget<'a> {
    // NOTE ALL EVENT TARGETS MUST BE SEPERATE! Because they will all have mut refs
    // for each one in a seperate thread. so for example need to have seperate locationItemTarget
    // and locationCreatures target even though they modify the same mapLocation, they also then unique uid
    LocationItemTarget(&'a mut Vec<Item>, UID),
    LocationCreaturesTarget(&'a mut CreatureList, UID),
    CreatureTarget(&'a mut CreatureState),
}
impl EventTarget<'_> {
    fn get_id(&self) -> UID {
        match &self {
            EventTarget::LocationItemTarget(_, id) => *id,
            EventTarget::LocationCreaturesTarget(_, id) => *id,
            EventTarget::CreatureTarget(c) => c.components.id_component.id(),
            // TODONEXT: Add new EventTarget BattleList
        }
    }
}

pub struct Event {
    pub event_type: EventType,
    pub get_requirements: Box<fn(&EventTarget, &EventType) -> bool>,
    pub on_fail: Option<EventChain>,
    pub target: UID,
}
impl Event {
    pub fn mutate(self, effected: &mut EventTarget) -> Option<Event> {
        match self.event_type {
            EventType::RemoveCreature(id, next_op, current_frame) => match effected {
                EventTarget::LocationCreaturesTarget(v, _) => {
                    let rmed = v.drain_specific_creature(id, current_frame);
                    if let Some(next) = next_op {
                        return Some(Event {
                            event_type: EventType::AddCreature(rmed, current_frame),
                            get_requirements: Box::new(|_, _| true),
                            on_fail: None,
                            target: next,
                        });
                    } else {
                        return None;
                    }
                }
                _ => {
                    panic!("trying to remove creature wrong target");
                }
            },
            EventType::AddCreature(c, current_frame) => {
                match effected {
                    EventTarget::LocationCreaturesTarget(c_list, id) => {
                        c_list.add_creature(c, current_frame);
                    }
                    _ => {
                        panic!("trying to add creature wrong target");
                    }
                }
                return None;
            }
            EventType::RemoveItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        let mut found = false;
                        let mut zero_index = None;
                        let mut i = 0;
                        for v in v.iter_mut() {
                            if v.item_type == t {
                                v.quantity -= q;
                                found = true;
                                if v.quantity == 0 {
                                    zero_index = Some(i);
                                }
                            }
                            i += 1;
                        }
                        if found {
                            if let Some(ii) = zero_index {
                                v.remove(ii);
                            }
                            return None;
                        }
                        return None;
                    }
                    EventTarget::CreatureTarget(c) => {
                        let mut found = false;
                        let mut zero_index = None;
                        let mut i = 0;
                        for v in c.inventory.iter_mut() {
                            if v.item_type == t {
                                v.quantity -= q;
                                found = true;
                                if v.quantity == 0 {
                                    zero_index = Some(i);
                                }
                            }
                            i += 1;
                        }
                        if found {
                            if let Some(ii) = zero_index {
                                c.inventory.remove(ii);
                            }
                            return None;
                        }
                        return None;
                    }
                    _ => {
                        panic!("Got remove item for wrong target");
                    }
                }
                // panic!(format!("Failed to find item in event! event: {:#?}", &self));
            }
            EventType::AddItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        let mut inventory = v;
                        for v in inventory.iter_mut() {
                            if v.item_type == t {
                                v.quantity += q;
                                return None;
                            }
                        }
                        inventory.push(Item {
                            item_type: t,
                            quantity: q,
                        });
                        return None;
                    }
                    EventTarget::CreatureTarget(c) => {
                        for v in c.inventory.iter_mut() {
                            if v.item_type == t {
                                v.quantity += q;
                                return None;
                            }
                        }
                        c.inventory.push(Item {
                            item_type: t,
                            quantity: q,
                        });
                        return None;
                    }
                    _ => {
                        panic!("Got add item for wrong target");
                    }
                }
                // TODO: Panic if inv full?>
            }
            EventType::IterateBudding() => match effected {
                EventTarget::CreatureTarget(c) => {
                    let bud = c.components.budding_component.as_mut().unwrap();
                    bud.frame_ready_to_reproduce += bud.reproduction_rate as u128;
                    None
                }
                _ => panic!("Wrong event target for budding"),
            },
            // EventType::IterateMovement(current_frame) => {
            //     match effected {
            //         EventTarget::CreatureTarget(c) => {
            //             let movement = c.components.movement_component.as_mut().unwrap();
            //             let dst_reached =  c.components.location_component.location == movement.destination.position &&
            //             c.components.region_component.region == movement.destination.region;
            //             movement.check_ready_and_reset_move(current_frame, dst_reached);
            //             None
            //         },
            //         _ => panic!("Wrong event target for budding")
            //     }
            // }
            EventType::InitializeMovement(current_frame, destination) => match effected {
                EventTarget::CreatureTarget(c) => {
                    let movement = c.components.movement_component.as_mut().unwrap();
                    movement.set_new_destination(destination, current_frame);
                    None
                }
                _ => panic!("Wrong event target for budding"),
            },
            EventType::EnterBattle(battle_id) => match effected {
                EventTarget::CreatureTarget(c) => {
                    // TODONEXT: Set in_battle
                    return None
                },
                _ => {
                    panic!("Got enter battle for wrong target");
                }
            },
            EventType::LeaveBattle() => match effected {
                EventTarget::CreatureTarget(c) => {
                    let battlec = c.components.battle_component.as_mut().unwrap();
                    //TODONEXT: Set in_battle to none
                    return None
                },
                _ => {
                    panic!("Got leave battle for wrong target");
                }
            },
        }
    }
}
impl Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("event_type", &self.event_type)
            .field("target", &self.target)
            .finish()
    }
}

#[derive(Debug)]
pub enum EventType {
    RemoveCreature(UID, Option<UID>, u128), // first is what to remove, 2nd is where to add next if there is next
    AddCreature(CreatureState, u128),
    RemoveItem(u32, ItemType),
    AddItem(u32, ItemType),
    IterateBudding(),
    //IterateMovement(u128),
    InitializeMovement(u128, Location),
    EnterBattle(u128),
    LeaveBattle(), // Mostly for canceling battle events in case of conflict
    // TODONEXT: make AddBattle event.
}

pub fn process_events_from_mapstate(
    m: &mut MapState,
    event_chains: Vec<EventChain>,
    creature_list_targets: bool,
) {
    // get a mut ref to all creatures and locations?
    // note have to do it in a SINGLE LOOP because otherwise compiler gets confused with
    // multiple m.region mut refs. UGG
    let mut all_creature_targets: Vec<EventTarget> = m
        .regions
        .par_iter_mut()
        .flat_map(|x| {
            x.par_iter_mut().flat_map(|y| {
                y.grid.par_iter_mut().flat_map(|xl| {
                    xl.par_iter_mut().flat_map(|yl| {
                        let mut creatures = &mut yl.creatures;
                        let mut ret = Vec::new();
                        if creatures.holds_creatures() {
                            if creature_list_targets {
                                ret.push(EventTarget::LocationCreaturesTarget(
                                    creatures,
                                    yl.id_component_creatures.id(),
                                ));
                            } else {
                                let mut creatures = if let Some(cit) = creatures.get_par_iter_mut()
                                {
                                    let mut cc: Vec<EventTarget> =
                                        cit.map(|c| EventTarget::CreatureTarget(c)).collect();
                                    ret.extend(cc);
                                };
                            }
                        }
                        ret.push(EventTarget::LocationItemTarget(
                            &mut yl.items,
                            yl.id_component_items.id(),
                        ));
                        ret
                    })
                })
            })
        })
        .collect();

    let mut next = process_events(&mut all_creature_targets, event_chains);
    while next.len() > 0 {
        next = process_events(&mut all_creature_targets, next);
    }
}

pub fn process_events<'a, 'b>(
    targets: &'a mut Vec<EventTarget<'b>>,
    event_chains: Vec<EventChain>,
) -> Vec<EventChain> {
    let mut tasks_map: HashMap<UID, TaskList> = HashMap::new();
    let mut uid_map: HashMap<UID, &mut EventTarget<'b>> = HashMap::new();
    {
        for t in targets.iter_mut() {
            let id = match t {
                EventTarget::LocationItemTarget(_, id) => *id,
                EventTarget::CreatureTarget(c) => c.components.id_component.id(),
                EventTarget::LocationCreaturesTarget(_, id) => *id,
            };
            //println!("Adding id: {}", id);
            uid_map.insert(id, t);
        }
    }
    for ec in event_chains.into_iter() {
        let key = ec.events[0].target;
        println!("looking at target: {}", key);
        match tasks_map.get_mut(&key) {
            Some(tl) => {
                tl.tasks.push(ec);
            }
            None => {
                let m = uid_map.remove(&key).unwrap();
                let tl = TaskList {
                    target: m,
                    tasks: vec![ec],
                };
                tasks_map.insert(key, tl);
            }
        }
    }

    let mut task_lists = Vec::new();
    // Run task list, get back Next EventChain
    for (_, task_list) in tasks_map.drain() {
        task_lists.push(task_list);
    }

    let next: Vec<Option<EventChain>> = task_lists
        .into_par_iter()
        .flat_map(move |tl| tl.process())
        .collect();
    let mut next_no_option = Vec::new();
    for e in next {
        match e {
            Some(ee) => next_no_option.push(ee),
            None => {}
        }
    }
    next_no_option
}
