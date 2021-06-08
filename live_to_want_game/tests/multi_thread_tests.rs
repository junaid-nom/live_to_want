
extern crate rayon;
use std::{sync::Mutex, cell::RefCell, rc::Rc, sync::Arc};

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
fn test_chain_multithread_items() {
    let x: Vec<u32> = (0..100).collect();
    let y: i32 = x.into_par_iter().map(|_| {
        // make a mapstate with some deer
        let openr = RegionCreationStruct::new(5,5, 0, vec![]);
        let rgrid = vec![
            vec![openr.clone()],
        ];
        //create map
        let mut map = MapState::new(rgrid, 0);
        let  region: &mut MapRegion = &mut map.regions[0][0];

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
            events: vec!(pickup_fail),
        };
        let pickup_fail2 = Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|_, _| true),
            on_fail: None,
            target: deer2_id,
        };
        let event_fail2 = EventChain {
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
            events: vec![pickup1, remove1],
        };
        let deer_chain2 = EventChain {
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

#[test]
// TODONEXT: create a map. have three deer. have two deer at the same time declare attack on the same victim deer.
// check to make sure that only one battle occurs. only 2 units in battle at a time.
// then also check to make sure the battle actually finishes with the expected result: one deer dead, the other with the first deers items
fn test_chain_multithread_battle<'a>() {
    // Below is bs copy pasted... needs to be redone
    let x: Vec<u32> = (0..100).collect();
    let y: i32 = x.into_par_iter().map(|_| {
        // make a mapstate with some deer
        let openr = RegionCreationStruct::new(5,5, 0, vec![]);
        let rgrid = vec![
            vec![openr.clone()],
        ];
        //create map
        let mut map = MapState::new(rgrid, 0);
        let  region: &mut MapRegion = &mut map.regions[0][0];

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

        let mut deer3 = CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        deer3.components.location_component = LocationComponent {
            location: Vu2{x: 1, y: 1}
        };
        deer3.inventory.push(Item{
            item_type: ItemType::Berry,
            quantity: 1,
        });
        
        let deer1_id = deer1.components.id_component.id();
        let deer2_id = deer2.components.id_component.id();
        let deer3_id = deer3.components.id_component.id();
        region.grid[1][1].creatures.add_creature(
            deer1, 0
        );
        region.grid[1][1].creatures.add_creature(
            deer2, 0
        );
        region.grid[1][1].creatures.add_creature(
            deer3, 0
        );
        
        let mut root = GoalNode {
            get_want_local: Box::new(|_, _| 1),
            get_effort_local: Box::new(|_, _| 1),
            children: Vec::new(),
            name: "root",
            get_command: Some(Box::new(|m: & MapState, c| CreatureCommand::Attack("attack_deer_1", c, m.find_closest_creature_to_creature(c).unwrap(), m.battle_list.id))),
            get_requirements_met: Box::new(|_, _| true),
        };

        let game_state = GameState {
            map_state:map
        };
        run_frame(game_state, &root);

        
        assert_eq!(1, 1);
        1
    }).sum();
    assert_eq!(y, 100);
}



