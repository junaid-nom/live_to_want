use fmt::Debug;

use crate::{map_state::Item, utils::UID, creature::CreatureState, map_state::ItemType, map_state::MapState};
use std::collections::HashMap;
use core::fmt;
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
    pub index: usize,
    pub events: Vec<Event>,
}
impl EventChain {
    fn process(self, effected: &mut EventTarget) -> Option<EventChain> {
        let e = &self.events[*&self.index];
        let success = (*e.get_requirements)(&*effected, &e.event_type);
        if success {
            let added_event = e.mutate(effected);
            let mut se = self;
            se.index+=1;
            if let Some(e) = added_event {
                se.events.insert(se.index, e);
            }
            if se.events.len() > se.index {
                Some(se)
            }
            else {
                None
            }
        } else {
            let mut e = self;
            e.events.remove(e.index).on_fail
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum EventTarget<'a> {
    // NOTE ALL EVENT TARGETS MUST BE SEPERATE! Because they will all have mut refs
    // for each one in a seperate thread. so for example need to have seperate locationItemTarget
    // and locationCreatures target even though they modify the same mapLocation, they also then unique uid
    LocationItemTarget(&'a mut Vec<Item>, UID),
    LocationCreaturesTarget(&'a mut Vec<CreatureState>, UID),
    CreatureTarget(&'a mut CreatureState),
}
impl EventTarget<'_> {
    fn get_id(&self) -> UID {
        match &self {
            EventTarget::LocationItemTarget(_, id) => {*id}
            EventTarget::LocationCreaturesTarget(_, id) => {*id}
            EventTarget::CreatureTarget(c) => {c.components.id_component.id()}
        }
    }
}

pub struct Event {
    pub event_type: EventType,
    pub get_requirements: Box<fn (&EventTarget, &EventType) -> bool>,
    pub on_fail: Option<EventChain>,
    pub target: UID,
}
impl Event {
    pub fn mutate(&self, effected: &mut EventTarget) -> Option<Event> {
        match &self.event_type {
            EventType::RemoveCreature(id, next_op) => {
                match effected {
                    EventTarget::LocationCreaturesTarget(v, _) => {
                        let to_rm = v.iter().position(|c: &CreatureState| {
                            c.components.id_component.id() != *id
                        }).unwrap();
                        let rmed = v.remove(to_rm);
                        if let Some(next) = next_op {
                            return Some(Event {
                                event_type: EventType::AddCreature(rmed),
                                get_requirements: Box::new(|_, _| true),
                                on_fail: None,
                                target: *next,
                            });
                        } else {
                            return None;
                        }
                    }
                    _ => { panic!("trying to remove creature wrong target"); }
                }
            },
            EventType::AddCreature(c) => {
                return None
            },
            EventType::RemoveItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        let mut found = false;
                        let mut zero_index = None;
                        let mut i = 0;
                        for v in v.iter_mut() {
                            if v.item_type == *t {
                                v.quantity -= q;
                                found = true;
                                if v.quantity == 0 {
                                    zero_index = Some(i);
                                }
                            }
                            i +=1;
                        }
                        if found {
                            if let Some(ii) = zero_index {
                                v.remove(ii);
                            }
                            return None
                        }
                        return None
                    }
                    EventTarget::CreatureTarget(c) => {
                        let mut found = false;
                        let mut zero_index = None;
                        let mut i = 0;
                        for v in c.inventory.iter_mut() {
                            
                            if v.item_type == *t {
                                v.quantity -= q;
                                found = true;
                                if v.quantity == 0 {
                                    zero_index = Some(i);
                                }
                            }
                            i+=1;
                        }
                        if found {
                            if let Some(ii) = zero_index {
                                c.inventory.remove(ii);
                            }
                            return None
                        }
                        return None
                    }
                    _ => {
                        panic!("Got remove item for wrong target");
                    }
                    
                }
                panic!(format!("Failed to find item in event! event: {:#?}", &self));
            }
            EventType::AddItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        let mut inventory = v;
                        for v in inventory.iter_mut() {
                            if v.item_type == *t {
                                v.quantity += q;
                                return None;
                            }
                        }
                        inventory.push(Item{
                            item_type: *t,
                            quantity: *q,
                        });
                        return None
                    }
                    EventTarget::CreatureTarget(c) => {
                        for v in c.inventory.iter_mut() {
                            if v.item_type == *t {
                                v.quantity += q;
                                return None;
                            }
                        }
                        c.inventory.push(Item{
                            item_type: *t,
                            quantity: *q,
                        });
                        return None
                    }
                    _ => {
                        panic!("Got add item for wrong target");
                    }
                }
                // TODO: Panic if inv full?>
            }
            EventType::IterateBudding() => {
                match effected {
                    EventTarget::CreatureTarget(c) => {
                        let bud = c.components.budding_component.as_mut().unwrap();
                        bud.frame_ready_to_reproduce += bud.reproduction_rate as u128;
                        None
                    },
                    _ => panic!("Wrong event target for budding")
                }
            }
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
    RemoveCreature(UID, Option<UID>), // first is what to remove, 2nd is where to add next if there is next
    AddCreature(CreatureState),
    RemoveItem(u32, ItemType),
    AddItem(u32, ItemType),
    IterateBudding(),
}

pub fn process_events_from_mapstate (m: &mut MapState, event_chains: Vec<EventChain>) {
    // get a mut ref to all creatures and locations?
    // note have to do it in a SINGLE LOOP because otherwise compiler gets confused with
    // multiple m.region mut refs. UGG
    let mut all_creature_targets : Vec<EventTarget> = m.regions.par_iter_mut().flat_map(|x| {
        x.par_iter_mut().flat_map(|y| {
            y.grid.par_iter_mut().flat_map(|xl| {
                xl.par_iter_mut().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        let mut cc: Vec<EventTarget> = creatures.par_iter_mut().map(
                            |c| {
                            EventTarget::CreatureTarget(c)
                            }
                        ).collect();
                        cc.push(EventTarget::LocationItemTarget(&mut yl.items, yl.id_component_items.id()));
                        cc
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();

    let mut next = process_events(&mut all_creature_targets, event_chains);
    while next.len() > 0 {
        next = process_events(&mut all_creature_targets, next);
    }
}

pub fn process_events<'a, 'b>(targets: &'a mut Vec<EventTarget<'b>>, event_chains: Vec<EventChain>) -> Vec<EventChain> {
    let mut tasks_map: HashMap<UID, TaskList> = HashMap:: new();
    let mut uid_map: HashMap<UID, & mut EventTarget<'b>> = HashMap::new();
    {
        for t in targets.iter_mut() {
            let id = match t {
                EventTarget::LocationItemTarget(_, id) => {*id}
                EventTarget::CreatureTarget(c) => {c.components.id_component.id()}
                EventTarget::LocationCreaturesTarget(_, id) => {*id}
            };
            uid_map.insert(id, t);
        }
    }
    for ec in event_chains.into_iter() {
        let key = ec.events[ec.index].target;
        match tasks_map.get_mut(&key) {
            Some(tl) => {
                tl.tasks.push(ec);
            }
            None => {
                let m = uid_map.remove(&key).unwrap();
                let tl = TaskList {
                    target: m,
                    tasks: vec![ec]
                };
                tasks_map.insert(key, tl);
            }
        }
    }

    let mut task_lists =  Vec::new();
    // Run task list, get back Next EventChain
    for (_, task_list) in tasks_map.drain() {
        task_lists.push(task_list);
    }

    let next: Vec<Option<EventChain>> = task_lists.into_par_iter().flat_map(move |tl| tl.process()).collect();
    let mut next_no_option = Vec::new();
    for e in next {
        match e {
            Some(ee) => next_no_option.push(ee),
            None => {},
        }
    }
    next_no_option
}

