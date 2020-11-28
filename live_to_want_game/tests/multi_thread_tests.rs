
extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;
#[test]
fn test_rayon() {
    pub struct TaskListTest<'a> {
        ev: Vec<EventTarget<'a>>,
    }
    pub struct TaskListTest2<'a> {
        ev: Vec<EventTarget<'a>>,
        op: Option<EventTarget<'a>>
    }
    pub struct TaskListTest3<'a> {
        ev: Vec<EventTarget<'a>>,
        op: Option<EventTarget<'a>>,
        re: &'a mut EventTarget<'a>,
    }
    pub struct TaskListTest4<'a> {
        ev: Vec<EventTarget<'a>>,
        op: Option<EventTarget<'a>>,
        rc: Rc<EventTarget<'a>>,
    }
    pub struct TaskListTest5 {
        b: Box<u32>,
    }
    pub struct TaskListTest6 {
        b: Box<dyn Fn() -> bool>,
    }
    pub struct TaskListTest7 {
        b: Box<fn() -> bool>,
    }
    pub struct TaskListTest8 {
        b: Box<Box<dyn Fn() -> bool>>,
    }
    

    let mut v = Vec::new();
    let ev = vec!(EventTarget::LocationItemTarget(&mut v, 1));
    ev.into_par_iter().map(|x| x);

    let ev = vec![Rc::new(RefCell::new(2))]; // wont work

    let ev = vec![2];
    ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest {
        ev: Vec::new()
    }];
    ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest2 {
        ev: Vec::new(),
        op: None
    }];
    ev.into_par_iter().map(|x| x);

    let mut v = Vec::new();
    let mut eve = EventTarget::LocationItemTarget(&mut v, 1);
    let ev = vec![TaskListTest3 {
        ev: Vec::new(),
        op: None,
        re: &mut eve,
    }];
    ev.into_par_iter().map(|x| x);

    let mut eve = EventTarget::LocationItemTarget(&mut v, 1);
    let ev = vec![TaskListTest4 {
        ev: Vec::new(),
        op: None,
        rc: Rc::new(eve)
    }]; // doesnt work
    //ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest5{
        b: Box::new(5),
    }];
    ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest6{
        b: Box::new(|| false),
    }]; // DOESNT WORK! Fucking dyn!
    //ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest8{
        b: Box::new(Box::new(|| false)),
    }]; // DOESNT WORK! Fucking dyn!
    //ev.into_par_iter().map(|x| x);

    let ev = vec![TaskListTest7{
        b: Box::new(|| false),
    }]; // DOESNT WORK! Fucking dyn!
    ev.into_par_iter().map(|x| x);

    let evl = vec![Event {
        event_type: EventType::RemoveItem(3, ItemType::Berry),
        target: 1,
        get_requirements: Box::new(|_, _| false),
        on_fail: None,
    }]; // DOESNT WORK!!! Probably cause of the Box
    //evl.into_par_iter().map(|x| x);

    let mut eve = EventTarget::LocationItemTarget(&mut v, 1);
    let evc = vec![EventChain {
        index: 0,
        events: Vec::new(),
    }]; // doesnt work
    //evc.into_par_iter().map(|x| x);

    let vec_tl = vec![TaskList{
        target:&mut eve,
        tasks: Vec::new(),
    }]; // doesnt work...
    //vec_tl.into_par_iter().map(|x| x);
}


#[test]
fn test_chain_multithread() {
    let x: Vec<u32> = (0..100).collect();
    let y: i32 = x.into_par_iter().map(|_| {
        // make a mapstate with some deer
        let mut region = MapRegion{
            grid:Vec::new(),
            last_frame_changed: 0,
        };
        for x in 0..10 {
            let mut xList  = Vec::new();
            for y in 0..10 {
                let loc = MapLocation{
                    id_component_items: IDComponent::new(),
                    id_component_creatures: IDComponent::new(),
                    location: Vu2{x, y},
                    creatures: CreatureList::new(true, 0),
                    items: Vec::new(),
                    is_exit: false,
                };
                xList.push(loc);
            }
            region.grid.push(xList);
        }
    
        let mut deer1 = CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        deer1.components.location_component = LocationComponent {
            location: Vu2{x: 1, y: 1}
        };
    
        let mut deer2 =CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        deer2.components.location_component = LocationComponent {
            location: Vu2{x: 1, y: 1}
        };
        let deer1_id = deer1.components.id_component.id();
        let deer2_id = deer2.components.id_component.id();
        region.grid[1][1].creatures.add_creature(
            deer1, 0
        );
        region.grid[1][1].creatures.add_creature(
            deer2, 0
        );
        region.grid[1][1].items.push(Item{
            item_type: ItemType::Berry,
            quantity: 1,
        });
        let berry_id = region.grid[1][1].id_component_items.id();

        let loc = &mut region.grid[1][1];
        let mut iter_mut = loc.creatures.get_iter_mut().unwrap();
        let d1_ref = iter_mut.next().unwrap();
        let d2_ref = iter_mut.next().unwrap();
        let loc_ref = &mut loc.items;

        // let d1_ref = &mut region.grid[1][1].creatures[0];
        // let d2_ref = &mut region.grid[1][1].creatures[1];
        // let loc_ref = &mut region.grid[1][1].items;
        
        // make some event chain examples
        // pick up item -> remove item (if fail remove item again) (note, in rl would do reverse)
        let pickup1 = Event {
            event_type: EventType::AddItem(1, ItemType::Berry),
            get_requirements: Box::new(|_, _| true),
            on_fail: None,
            target: deer1_id,
        };
        let pickup2 = Event {
            event_type: EventType::AddItem(1, ItemType::Berry),
            get_requirements: Box::new(|_, _| true),
            on_fail: None,
            target: deer2_id,
        };
        let pickup_fail = Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|_, _| true),
            on_fail: None,
            target: deer1_id,
        };
        let event_fail1 = EventChain {
            index: 0,
            events: vec!(pickup_fail),
        };
        let pickup_fail2 = Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|_, _| true),
            on_fail: None,
            target: deer2_id,
        };
        let event_fail2 = EventChain {
            index: 0,
            events: vec!(pickup_fail2),
        };
        let remove1=  Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|e, _| {
                match e {
                    EventTarget::LocationItemTarget(i, _) => {
                        for item in i.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    EventTarget::CreatureTarget(c) => {
                        for item in c.inventory.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    _ => {
                        panic!("Got wrong target for remove item ev");
                    }
                }
            }),
            on_fail: Some(event_fail1),
            target: berry_id
        };
        let remove2=  Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|e, _| {
                match e {
                    EventTarget::LocationItemTarget(i, _) => {
                        for item in i.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    EventTarget::CreatureTarget(c) => {
                        for item in c.inventory.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    _ => {
                        panic!("Got wrong target for remove item ev");
                    }
                }
            }),
            on_fail: Some(event_fail2),
            target: berry_id
        };
    
        let deer_chain1 = EventChain {
            index: 0,
            events: vec![pickup1, remove1],
        };
        let deer_chain2 = EventChain {
            index: 0,
            events: vec![pickup2, remove2],
        };
    
        // for all events, get current target, and make hashtable of Vec for it
        // transfer the Vec and Targets to a TaskList
        let event_chains = vec![deer_chain1, deer_chain2];
        let mut ed1 = EventTarget::CreatureTarget(d1_ref);
        let mut ed2 = EventTarget::CreatureTarget(d2_ref);
        let mut eloc = EventTarget::LocationItemTarget(loc_ref, berry_id);
        let mut targets = vec![ed1, ed2, eloc];
        //let targets = &mut targets;
        
        let mut next = process_events(&mut targets, event_chains);
        while next.len() > 0 {
            next = process_events(&mut targets, next);
        }
        assert_eq!(next.len(), 0);
        assert_eq!(region.grid[1][1].items.len(), 0);
        let total: u32 = region.grid[1][1].creatures.get_par_iter().unwrap().map(|c| {
            let ret: u32 = c.inventory.iter().map(|i| i.quantity).sum();
            ret
        }).sum();
        assert_eq!(total, 1);
        1
    }).sum();
    assert_eq!(y, 100);
}

