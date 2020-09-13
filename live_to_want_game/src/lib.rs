

use std::cmp::Ordering;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fmt::{Debug, Formatter}, borrow::Borrow};
use std::sync::{Arc, atomic::AtomicU64};
use core::fmt;
use rand::prelude::*;

extern crate rayon;
use rayon::prelude::*;

// NOTE: All event chains with items need to end in a final failure case of putting item on ground.
// this is because you can try to give an item away as someone else fills your inventory and 
// if both giving fails and putting back into your inventory fails, need to put item somewhere, so put on ground.

// TODO: 
// Pretty sure items in a MapLocation and inventory in a creature state don't have to be rc<refcell<>>

static COUNTER: AtomicU64 = AtomicU64::new(1); // TODO: Upgrade to a 128 bit one when it comes out of nightly build
fn get_id() -> u64 { COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }
type UID = u64;

// game constants:
static STARVING_SLOW_METABOLISM_FACTOR: f32 = 0.5;


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

trait Component {
    fn get_visible() -> bool {
        false
    }
}

#[derive(Default)]
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct ComponentMap {
    id_component: IDComponent,
    health_component: Option<HealthComponent>,
    location_component: LocationComponent,
    region_component: RegionComponent,
    name_component: Option<NameComponent>,
    creature_type_component: Option<CreatureTypeComponent>,
    starvation_component: Option<StarvationComponent>,
    block_space_component: Option<BlockSpaceComponent>,
    movement_component: Option<MovementComponent>,
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
struct IDComponent {
    id: UID,
}
impl IDComponent {
    fn new() -> IDComponent{
        IDComponent{
            id: get_id()
        }
    }
}
impl Default for IDComponent {
    fn default() -> Self {
        IDComponent::new()
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
struct HealthComponent {
    health: i32,
    max_health: i32,
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Default, Debug, Hash, PartialEq, Eq)]
struct LocationComponent {
    location: Vector2,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
struct RegionComponent {
    region: Vector2,
}
impl Component for RegionComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct NameComponent {

}
impl Component for NameComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct CreatureTypeComponent {

}
impl Component for CreatureTypeComponent {
    fn get_visible() -> bool {
        true
    }
}

// TODO: Either:
// 1. If you have BlocKSpaceComponent you CANNOT have a move component
// or 2. Blockers can move, but there needs to be a special EXTRA entire loop
// check in run_frame where any collisions with blocks and other creatures, the 
// other creatures have to be moved the nearest open space, and if the colliding
// creature is a blocker as well then it has to move to an unoccupied space? and this must be done LINEARLY
// because u cud have 2-4 blockers all moving to the same space

// to allow AI to eventually get smart enough to figure out that they should
// chop down trees to make navigation easier gonna add a "breakable" bool here.
// ACTUALLY I wont cause that should be based on like health component existing
// or something like that
#[derive(Debug, Hash, PartialEq, Eq)]
struct BlockSpaceComponent {
}
impl Component for BlockSpaceComponent {
    fn get_visible() -> bool {
        true
    }
}

// Events set the Navigation Component.
// Nagivation system then will set this MovementComponent system
// Movement system will then run, where it will check frame_ready_to_move and if
// its ready, create an event chain to move obj
// DONE: Okay so I need to CHANGE the event-chain system to also be able to RETURN
// a new event chain dynamically created! That has a new Event type that OWNS a creaturestate
// that is then meant to be moved into a new location (spawn will be similar)
#[derive(Hash, Debug, PartialEq, Eq)]
pub struct MovementComponent {
    speed: usize,
    destination: Location,
    cached_navigation: Vec<Location>,
    cache_last_updated_frame: u128,
    navigating: bool,
    frame_ready_to_move: u128, // essentially if frame_ready to move is the current frame or earlier, move to destination
}
impl Component for MovementComponent {
    fn get_visible() -> bool {
        true
    }
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
struct StarvationComponent {
    calories: i32,
    metabolism: usize,
}
impl Component for StarvationComponent {
    fn get_visible() -> bool {
        true
    }
}

// pub enum CreatureComponents {
//     Health(i32),
//     Location(Location),
//     Region(Location),
//     Name(String),
//     CreatureType(CreatureType),
// }

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Item {
    pub item_type: ItemType,
    pub quantity: u32,
}

// TODO: GET RID OF ALL THESE FUCKING attribute type FIELDS
// Instead make a big enum of "Components"
// Components have a func "get_is_visible()"
// Components are in a Rc<RefCell<>> so that they can be also added to a big HashTable
// big hashtable should only have a WEAK reference and remove it self if the thing dies (save index and remove highest index first)
// so u can do stuff like for every Metabolism component, subtract calories or something
#[derive(Debug)]
#[derive(Hash, PartialEq, Eq)]
pub struct CreatureState {
    pub components: ComponentMap,
    pub memory: CreatureMemory,
    pub inventory: Vec<Item>,
}
impl CreatureState {
    fn new<'a>(loc: Vector2) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = LocationComponent{location:loc};
        ret
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
        let mut fString = String::new();
        for item in &self.inventory {
            fString = format!("{},{}",fString, item.quantity);
        }
        write!(f, "{}", fString)
    }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq)]
pub struct CreatureMemory {
    
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Vector2 {
    x: i32,
    y: i32,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapState {
    regions: Vec<Vec<MapRegion>>,
    frame_count: u128,
    navigation_map: NavigationMap,
}

#[derive(Debug)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    region: Vector2,
    location: Vector2,
}

#[derive(Debug)]
#[derive(Default)]
pub struct NavRegion {
    grid: Vec<Vec<NavPoint>>,
    region_distances: Vec<Vec<u32>>,
    last_frame_updated: u128,
    left: bool,
    right: bool,
    up: bool, // TODO: This kinda doesnt make sense maybe need dist between regions idk?
    down: bool,
}

#[derive(Debug)]
#[derive(Default)]
pub struct NavPoint {
    blocked: bool,
    point_distances: Vec<Vec<u32>>,
    is_exit: ExitPoint
}

#[derive(Debug)]
enum ExitPoint {
    None,
    Left,
    Right,
    Up,
    Down,
}
impl Default for ExitPoint {
    fn default() -> Self {
        ExitPoint::None
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct NavigationMap {
    map: Vec<Vec<NavRegion>>,
}
impl NavigationMap {
    fn update(&mut self, region: Vector2) {
        // update the navRegion
        
        // if the left/right/up/down access changes then update all the region_distances

        // PANIC if 1. exit node has a 
    }

    fn navigate_to(&mut self, start: &Location, goal: &Location) -> Vec<Location> {
        // Currently just using a simple algo that assumes there are NO blockers anywhere and in same region
        // TODO: make a VecVec VecVec of region(with last updated piece)->location->blocked. and then 
        // make a giant cached navigation thing FOR EACH point...
        // will get weird cause if u change the viable entrance/exits of regions it would mean needing to change the
        // between region map as well.
        // Need to also teach AI how to like "break" things to create shorter path?
        let mut ret = Vec::new();
        if start.region == goal.region {
            let mut current_loc = start.location;
            while current_loc != goal.location {
                let xchange = 
                    if current_loc.x > goal.location.x { -1 } 
                    else if current_loc.x < goal.location.x { 1 }
                    else { 0 };
                let ychange = 
                    if current_loc.y > goal.location.y { -1 } 
                    else if current_loc.y < goal.location.y { 1 }
                    else { 0 };
                if xchange == 0 { current_loc.y += ychange; } else if ychange == 0 { current_loc.x += xchange; } 
                    else {
                        if rand::random() {
                            current_loc.x += xchange;
                        } else {
                            current_loc.y += ychange;
                        }
                    };
                ret.push(Location{region:start.region, location: current_loc});
            }
        } else {
            panic!("Havent implemented cross-region navigation yet");
        }
        ret
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    grid: Vec<Vec<MapLocation>>,
    last_frame_changed: u128, // if nav system last updated before this frame, update it
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation {
    id_component_items: IDComponent,
    id_component_creatures: IDComponent,
    location: Vector2,
    is_exit: bool, // exits should not be allowed to have creatures placed on them. also they must not have a block INBETWEEN them.
    creatures: Option<Vec<CreatureState>>, // some locations will be perma blocked and no creatures allowed
    items: Vec<Item>,
}
impl MapLocation {
    fn get_if_blocked(&self) -> bool {
        if let Some(creatures) = self.creatures.as_ref() {
            for c in creatures.iter() {
                if let Some(_) = c.components.block_space_component {
                    return true
                }
            }
        } else {
            return true;
        }
        
        false
    }
}

#[derive(Debug)]
pub enum CreatureCommand<'b>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'static str, &'b CreatureState, Vector2),
    Chase(&'static str, &'b CreatureState, &'b CreatureState),
    Attack(&'static str, &'b CreatureState, &'b CreatureState),
    TakeItem(&'static str, InventoryHolder<'b>, InventoryHolder<'b>, Item),
}
impl CreatureCommand<'_> {
    pub fn to_event_chain(&self) -> Option<EventChain> {
        match self {
            CreatureCommand::MoveTo(_, _, _) => {}
            CreatureCommand::Chase(_, _, _) => {}
            CreatureCommand::Attack(_, _, _) => {}
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
                    index: 0,
                    events: vec![remove, add],
                })
            }
        }
        None
    }
}

fn get_id_from_inventory(inv: &InventoryHolder) -> UID {
    match inv {
        InventoryHolder::CreatureInventory(c) => {c.components.id_component.id}
        InventoryHolder::LocationInventory(l) => {l.id_component_items.id}
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

pub struct GoalConnection<'a> {
    pub child: Arc<GoalNode<'a>>,
    pub is_additive: bool,
    pub amplifier: f32,
}

/// Is a list of all events for that target for a given frame cycle
/// Must place all tasks for that target in here at once or could cause race conditions
//#[derive(std::marker::Sized)] doesnt work...
pub struct TaskList<'a, 'b> {
    target: &'a mut EventTarget<'b>,
    tasks: Vec<EventChain>,
}
impl TaskList<'_, '_> {
    fn process(mut self) -> Vec<Option<EventChain>> {
        let mut ret = Vec::new();
        for task in self.tasks.into_iter() {
            ret.push(task.process(&mut self.target));
        }
        ret
    }
}



#[derive(Debug)]
pub struct EventChain {
    index: usize,
    events: Vec<Event>,
}
impl EventChain {
    fn process(self, effected: &mut EventTarget) -> Option<EventChain> {
        let e = &self.events[*&self.index];
        let success = e.get_requirements.deref()(&*effected, &e.event_type);
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
            EventTarget::CreatureTarget(c) => {c.components.id_component.id}
        }
    }
}

pub struct Event {
    event_type: EventType,
    get_requirements: Box<fn (&EventTarget, &EventType) -> bool>,
    on_fail: Option<EventChain>,
    target: UID,
}
impl Event {
    fn mutate(&self, effected: &mut EventTarget) -> Option<Event> {
        match &self.event_type {
            EventType::RemoveCreature(id, next_op) => {
                match effected {
                    EventTarget::LocationCreaturesTarget(v, _) => {
                        let to_rm = v.iter().position(|c: &CreatureState| {
                            c.components.id_component.id != *id
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
}


pub struct GoalCacheNode<'a> {
    goal: &'a GoalNode<'a>,
    children: Option<Vec<Rc<RefCell<GoalCacheNode<'a>>>>>,
    want_local: u32,
    effort_local: u32,
    requirement_met: bool,
    motivation_global: Option<f32>, // if None is not calculated yet. Should be (sum(want/effort for all children) + local want) / local effort
}
impl GoalCacheNode<'_> {
    fn new<'a>(goal: &'a GoalNode, map_state :&MapState, c_state : &CreatureState) -> GoalCacheNode<'a> {
        let new = GoalCacheNode {
            goal,
            children: None,
            want_local: (goal.get_want_local)(map_state, c_state),
            effort_local: (goal.get_effort_local)(map_state, c_state),
            requirement_met: (goal.get_requirements_met)(map_state, c_state),
            motivation_global: None,
        };
        new
        // NOTE: Could make an outer struct "GoalCacheNetwork", that holds a root_node and the existing_cache and auto create network?
    }

    fn _my_func(num: i32, list_of_nums: &mut Vec<i32>) {
        list_of_nums.push(num);
        if num - 1 >= 0 {
            GoalCacheNode::_my_func(num - 1, list_of_nums);
        }
    }

    fn _my_fc<'a>() -> Option<MapState> {
        let poop : Option<MapState>;
        poop = Some(MapState::default());
        // MUST USE & IN FRONT OF OPTION SO IT DOESNT GET TAKEN!
        let _p: Option<MapState> = match poop.as_ref() {
            Some(_n) => None,
            None => None
        };
        match poop {
            Some(_n) => None,
            None => None
        }
    }

    fn setup_children<'a>(goal_cache:  Rc<RefCell<GoalCacheNode<'a>>>, map_state :&MapState, c_state : &CreatureState, existing_caches: Rc<RefCell<HashMap<&'a str, Rc<RefCell<GoalCacheNode<'a>>>>>>) {
        let goal_cache = goal_cache.clone();
        let mut goal_cache = goal_cache.borrow_mut();
        if let Some(_) = goal_cache.children {
            // this node is setup already
            return;
        } else {
            goal_cache.children = Some(Vec::new());
            for child_goal in &goal_cache.goal.children {
                let existing_caches = existing_caches.clone();
                let cref: Rc<RefCell<GoalCacheNode<'a>>> = {
                    let mut existing_cache_ref = existing_caches.borrow_mut();
                    let entry = existing_cache_ref.entry(child_goal.child.name).or_insert(
            {
                        Rc::new(RefCell::new(GoalCacheNode::new(child_goal.child.borrow(), map_state, c_state)))
                    });
                    entry.clone()
                }; // need to drop borrow_mut before next call
                GoalCacheNode::setup_children( cref.clone(), map_state, c_state, existing_caches.clone());
                // is always true...
                if let Some(children) = &mut goal_cache.children {
                    children.push(cref.clone());
                } else {
                    panic!("This should never happen");
                }
            }
        }
    }

    fn get_connection_by_name<'a>(goal_parent: &'a GoalNode, child_name: &str) -> Option<&'a GoalConnection<'a>> {
        for c in &goal_parent.children {
            if c.child.deref().name == child_name {
                return Some(c);
            }
        }
        None
    }

    //note must call setup_children first
    fn setup_global_stats(goal_cache:  Rc<RefCell<GoalCacheNode>>, map_state :&MapState, c_state : &CreatureState) {
        let goal_cache_c = goal_cache.clone();
        let mut goal_cache = goal_cache_c.deref().borrow_mut();
        if let Some(_) = goal_cache.motivation_global {
            return
        } else {
            let mut sum_motivation: f32 = 0.0;
            let mut best_motivation:f32 = 0.0;
            // Essentially, all additive connections add together
            // example if u loot a deer u get both 10 meat and 10 bone, so additive
            // but for the meat itself, you can EITHER eat it or sell it, so its not additive
            // you can have a couple additive and a couple non-additive too
            // but u cant have 2 different kinds of additive (instead make a node that has them as additive children for that)
            if let Some(children) = &goal_cache.children {
                for c in children {
                    let need_setup = {
                        //let c_ref = c.clone();
                    
                        if let None = c.deref().borrow().motivation_global.as_ref() {
                            true
                        } else {
                            false
                        }
                    };
                    if need_setup {
                        GoalCacheNode::setup_global_stats(c.clone(), map_state, c_state);
                    }
                    let conn = GoalCacheNode::get_connection_by_name(goal_cache.goal, c.deref().borrow().goal.name).unwrap();
                    
                    let total_mot = c.deref().borrow().motivation_global.as_ref().unwrap() * conn.amplifier;
                    println!("total_mot {} amp {}", total_mot, conn.amplifier);
                    if conn.is_additive {
                        sum_motivation += total_mot;
                    } else {
                        if best_motivation < total_mot {
                            best_motivation = total_mot;
                        }
                    }
                }
            }
            
            //let mut goal_cache = goal_cache_c.deref().borrow_mut();
            println!("{} sum {} best {}", goal_cache.goal.name, sum_motivation, best_motivation);
            if best_motivation < sum_motivation {
                best_motivation = sum_motivation;
            }
            println!("{} true best {}", goal_cache.goal.name, best_motivation);
            goal_cache.motivation_global = Some((best_motivation + goal_cache.want_local as f32) / (goal_cache.effort_local as f32));
            println!("{} final {}", goal_cache.goal.name, goal_cache.motivation_global.as_ref().unwrap());
        }
    }

    pub fn get_final_command<'a, 'b>(goal_node: &'a GoalNode, map_state :&MapState, c_state : &'b CreatureState) -> Option<CreatureCommand<'b>> { 
        let parent = Rc::new(RefCell::new(GoalCacheNode::new(goal_node, map_state, c_state)));
        let existing_caches: Rc<RefCell<HashMap<&str, Rc<RefCell<GoalCacheNode>>>>> = Rc::new(RefCell::new(HashMap::new()));
        GoalCacheNode::setup_children(parent.clone(), map_state, c_state, existing_caches);
        GoalCacheNode::setup_global_stats(parent.clone(), map_state, c_state);

        // now go through the tree. if requirements met, go into it, if not ignore it. Find best
        // Node. then run the command function on that node.
        let mut to_visit : Vec<Rc<RefCell<GoalCacheNode>>> = Vec::new();
        let mut visited : usize = 0;
        //let b= parent.deref().borrow();
        //let c: Ref<GoalCacheNode> = parent.borrow(); // this only works if u uncomment use std::borrow:Borrow
        to_visit.push(parent.clone());
        let mut best_node : Option<Rc<RefCell<GoalCacheNode>>> = None;

        while to_visit.len() - visited > 0 {
            let look_at = to_visit[visited].clone();
            let look_at  = look_at.deref().borrow();
            let actionable  = match look_at.goal.get_command {
                Some(_) => true,
                None => false
            };
            let req_met = look_at.requirement_met;

            // NOTE, children of a node can have higher motivation!
            // A child can also have requirements met even if parent doesn't

            // Example: Looting dead deer met, which is child of attack deer.
            // No deer around so cant attack deer, but can loot it
            // looting dead deer is low effort so is way higher motivation too

            // so need to look at ALL NODES basically always
            // they can only be a "best node" if they are actionable and req met though
            if actionable && req_met {
                match best_node {
                    Some(n) => {
                        if look_at.motivation_global >= n.deref().borrow().motivation_global {
                            best_node = Some(to_visit[visited].clone());
                        } else {
                            best_node = Some(n);
                        }
                    },
                    None => {
                        best_node = Some(to_visit[visited].clone());
                    }
                }
            }
            if let Some(children) = &look_at.children {
                for c in children {
                    let c_ref = c.deref().borrow();
                    if !(to_visit.iter().any(|ch| ch.deref().borrow().goal.name == c_ref.goal.name)) {
                        to_visit.push(c.clone());
                    }
                }
            }
            visited+=1;
        }

        match best_node {
            Some(n) => {
                match &n.clone().deref().borrow().goal.get_command {
                    Some(f) => Some(f(map_state, c_state)),
                    None => None,
                }
                // Some((n.deref().borrow().goal.get_command).unwrap())
                // None
            },
            None => None
        }
    }
}

fn generate_goal_nodes<'a>() -> GoalNode<'a> {
    // TODO: Need to develop this
    let root = GoalNode {
        get_want_local: Box::new(|_, _| 0),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "root",
        get_command: None,
        get_requirements_met: Box::new(|_, _| false),
    };
    root
}

fn game() {
    // Make initial map state
    
    // generate initial goal root

    // start server

    // loop
    // get input from connections
    // run frame
    // if in super-fast mode, just loop
    // if in user controlled just check for input until receive something
    // also can do "slow" mode with a wait
}

fn starvation_system(c: &mut CreatureState) {
    if let Some(s) = c.components.starvation_component.as_mut() {
        if let Some(h) = c.components.health_component.as_mut() {
            let starving = s.calories <= 0;
            if starving {
                h.health -= 1;
            }
            let multiplier = if starving {STARVING_SLOW_METABOLISM_FACTOR} else {1.0};
            s.calories -= (s.metabolism as f32 * multiplier) as i32;
        } else {
            panic!("All starvation components require health component for: {}", c)
        }
    }
}


fn navigation_system(c: &mut CreatureState) {
    // TODO if the target for the creature is currently blocked, fuck
}

fn location_to_map_location<'a>(m: &'a MapState, location: &Location) -> &'a MapLocation {
    let region: &MapRegion = &m.regions[location.region.x as usize][location.region.y as usize];
    &region.grid[location.location.x as usize][location.location.y as usize]
}

fn movement_system(m: &MapState, c: &CreatureState) -> Option<EventChain> {
    if let Some(movement) = c.components.movement_component.as_ref() {
        if movement.frame_ready_to_move <= m.frame_count {
            let dest = location_to_map_location(m, &movement.destination).id_component_creatures.id;
            let rm_event = Event {
                event_type: EventType::RemoveCreature(c.components.id_component.id, 
                    Some(dest)),
                get_requirements: Box::new(|_,_| true),
                on_fail: None,
                target: dest,
            };
        }
    }
    None
}

fn run_frame(mut m: MapState, root: &GoalNode) -> MapState {
    {
        m.frame_count += 1;
    }
    // TODO: Maybe do something similar for every location and get 
    // event chains for stuff to mutate in every location and other upkeep stuff?

    // Run navigation system
    // get event chains from movement system and process them
    // Can run MUTABLE multiple systems here so far:
    // Starvation system
    // nav system
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        creatures.par_iter_mut().for_each(
                            |c| {
                                starvation_system(c);
                                navigation_system(c);
                            }
                        );
                    }
                })
            })
        })
    });

    // Can run immutable systems that rely on reading entire mapstate here
    let f_c = m.frame_count;
    let mov_op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_ref() {
                        let ret: Vec<Option<EventChain>> = creatures.par_iter().map(
                            |c| {
                                movement_system(&m, c)
                            }
                        ).collect();
                        return ret;
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();

    // TODO: Send out current map state to users via websocket
    // TODO: Then wait for them to respond if doing by-frame, or a timer

    // want to move THEN AFTER do ai stuff so ai can react to the movement
    let op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_ref() {
                        let ret: Vec<Option<EventChain>> = creatures.par_iter().map(
                            |c| {
                                match GoalCacheNode::get_final_command(&root, &m, &c) {
                                    Some(cc) => {cc.to_event_chain()}
                                    None => {None}
                                }
                            }
                        ).collect();
                        return ret;
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();
    let mut event_chains = Vec::new();
    for o in op_ecs {
        if let Some(ec) = o {
            event_chains.push(ec);
        }
    }

    process_events_from_mapstate(&mut m, event_chains);

    // Death system
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        creatures.retain(
                            |c| {
                                if let Some(h) = c.components.health_component.as_ref() {
                                    if h.health <= 0 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            }
                        );
                    }
                })
            })
        })
    });

    m
}

fn process_events_from_mapstate (m: &mut MapState, event_chains: Vec<EventChain>) {
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
                        cc.push(EventTarget::LocationItemTarget(&mut yl.items, yl.id_component_items.id));
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

fn process_events<'a, 'b>(targets: &'a mut Vec<EventTarget<'b>>, event_chains: Vec<EventChain>) -> Vec<EventChain> {
    let mut tasks_map: HashMap<UID, TaskList> = HashMap:: new();
    let mut uid_map: HashMap<UID, & mut EventTarget<'b>> = HashMap::new();
    {
        for t in targets.iter_mut() {
            let id = match t {
                EventTarget::LocationItemTarget(_, id) => {*id}
                EventTarget::CreatureTarget(c) => {c.components.id_component.id}
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

// NOTE I tried to make it not use Rc by make an vec
// that didn't work because you cant mutate elements to point
// to other elements of the same vec because bullshit
// Other solution could be to make the vec array, but then 
// all connections are integer indexs. to that list
// but then you need the children nodes to have immutable refs to the root node? which isn't possible?

// ACTUALLY can make it work if just have a new struct, that takes in each node invididually
// since the graph is basically static this is possible. See: tests::graph_without_vec_test
// though requires a lil unsafe
pub struct GoalNode<'a> {
    get_want_local: Box<fn(&MapState, &CreatureState) -> u32>,
    get_effort_local: Box<fn(&MapState, &CreatureState) -> u32>,
    children: Vec<GoalConnection<'a>>,
    name: &'a str,  // just for debugging really
    get_command: Option<Box<for<'f, 'c> fn(&MapState, &'f CreatureState) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of a organizing node
    get_requirements_met: Box<fn (&MapState, &CreatureState) -> bool>,
}

impl GoalNode<'_> {
}

#[cfg(test)]
mod tests {
    use crate::*;
    // use std::{cell::{RefCell}, rc::Rc};
    // use std::collections::HashMap;
    
    // extern crate rayon;
    // use rayon::prelude::*;

    use std::{cell::{Ref, RefCell}, rc::Rc};
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::{fmt::{Debug, Formatter}, borrow::Borrow};
    use std::sync::atomic::AtomicU64;
    use core::fmt;

    extern crate rayon;
    use rayon::prelude::*;

    #[test]
    fn iter_iter_par() {
        let x = vec![vec![1,2,3],vec![1,2,3],vec![1,2,3]];
        let new: Vec<i32> = x.par_iter().flat_map(|x| {
            let r: Vec<i32> = x.par_iter().map(|y| {
                y+1
            }).collect();
            r
        }).collect();
    }

    #[test]
    fn graph_without_vec_test() {
        pub struct Node<'a> {
            children: Vec<&'a Node<'a>>,
            my_num: u32,
        }
        impl Node<'_> {
            fn new<'a>(num: u32) -> Node<'a> {
                Node{
                    children: Vec::new(),
                    my_num: num,
                }
            }
        }
        pub struct NodeRoot<'a> {
            root: Node<'a>,
            left: Node<'a>,
            right: Node<'a>,
            child_both: Node<'a>,
            child_left: Node<'a>,
        }
        pub struct NodeWrapper<'a> {
            root_graph: NodeRoot<'a>,
        }

        fn make_node<'a>() -> NodeRoot<'a> {
            let mut node_root = NodeRoot {
                root: Node::new(0),
                left: Node::new(1),
                right: Node::new(2),
                child_both: Node::new(3),
                child_left: Node::new(4),
            };
            // need unsafe to self reference and return something
            unsafe {
                node_root.right.children.push(std::mem::transmute(&node_root.child_both));
                node_root.left.children.push(std::mem::transmute(&node_root.child_both));
                node_root.left.children.push(std::mem::transmute(&node_root.child_left));
                node_root.root.children.push(std::mem::transmute(&node_root.left));
                node_root.root.children.push(std::mem::transmute(&node_root.right));
            }
            
            node_root
        }
        let mut root = make_node();
        let new_c = Node::new(5);
        root.child_both = new_c;
        let wrap = NodeWrapper {
            root_graph: root,
        };
        let wrap2 = Box::new(wrap);

        assert_eq!(wrap2.deref().root_graph.root.children[0].children[0].my_num,5);
        assert_eq!(wrap2.deref().root_graph.root.children[0].children[1].my_num,4);
        // TODO: NOT SURE HOW TO BREAK THIS? But apparently it can be broken and is unsafe?
    }

    // PRETTY SURE GoalNode is fucked and needs Rc in connections to work
    // because if u return a GoalNode the connected other GoalNodes go out of scope
    fn generate_basic_graph() -> GoalNode<'static> {
        let mut root = GoalNode {
            get_want_local: Box::new(|_, _| 0),
            get_effort_local: Box::new(|_, _| 1),
            children: Vec::new(),
            name: "root",
            get_command: None,
            get_requirements_met: Box::new(|_, _| false),
        };
        let mut gather = GoalNode {
            get_want_local: Box::new(|_, _| 0),
            get_effort_local: Box::new(|_, _| 1),
            children: Vec::new(),
            name: "gather",
            get_command: None,
            get_requirements_met: Box::new(|_, _| false),
        };
        let mut hunt = GoalNode {
            get_want_local: Box::new(|_, _| 0),
            get_effort_local: Box::new(|_, _| 1),
            children: Vec::new(),
            name: "hunt",
            get_command: None,
            get_requirements_met: Box::new(|_, _| false),
        };

        // gather, normally these would lead to eat/sells but lazy for this test
        let berry = GoalNode {
            get_want_local: Box::new(|_, _| {
                100
            }),
            get_effort_local: Box::new(|_, c| {
                if c.components.location_component.location.x == 1 {
                    30
                } else {
                    50
                }
            }),
            children: Vec::new(),
            name: "berry",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("berry", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        let fruit = GoalNode {
            get_want_local: Box::new(|_, c| {
                if c.components.location_component.location.y == 1 {
                    101
                } else {
                    99
                }
            }),
            get_effort_local: Box::new(|_, c| {
                if c.components.location_component.location.x == 1 {
                    30
                } else {
                    50
                }
            }),
            children: Vec::new(),
            name: "fruit",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("fruit", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        gather.children.push(GoalConnection{
            child: Arc::new(berry),
            is_additive: false,
            amplifier: 1.0,
        });
        gather.children.push(GoalConnection{
            child: Arc::new(fruit),
            is_additive: false,
            amplifier: 1.0,
        });


        //hunt stuff
        let mut find_deer = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                50
            }),
            children: Vec::new(),
            name: "find_deer",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("find_deer", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        let mut attack_deer = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "attack_deer",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("attack_deer", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==5),
        };
        let mut loot_deer = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "loot_deer",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("loot_deer", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==6),
        };
        
        let eat = GoalNode {
            get_want_local: Box::new(|_, _| {
                10
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "eat",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("eat", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.y==0 && c.components.location_component.location.x==7),
        };
        let eat = Arc::new(eat);
        let sell = GoalNode {
            get_want_local: Box::new(|_, _| {
                10
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "sell",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("sell", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.y==1 && 
                (c.components.location_component.location.x==7 || c.components.location_component.location.x==11)),
        };
        let sell = Arc::new(sell);

        loot_deer.children.push(GoalConnection{
            child: sell.clone(),
            is_additive: true,
            amplifier: 4.0,
        });
        loot_deer.children.push(GoalConnection{
            child: eat.clone(),
            is_additive: true,
            amplifier: 7.0,
        });
        attack_deer.children.push(GoalConnection{
            child: Arc::new(loot_deer),
            is_additive: false,
            amplifier: 1.0,
        });
        find_deer.children.push(GoalConnection{
            child: Arc::new(attack_deer),
            is_additive: false,
            amplifier: 1.0,
        });
        hunt.children.push(GoalConnection{
            child: Arc::new(find_deer),
            is_additive: false,
            amplifier: 1.0,
        });


        let mut find_wolf = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                60
            }),
            children: Vec::new(),
            name: "find_wolf",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("find_wolf", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        let mut attack_wolf = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "attack_wolf",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("attack_wolf", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==9),
        };
        let mut loot_wolf = GoalNode {
            get_want_local: Box::new(|_, _| {
                0
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "loot_wolf",
            get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("loot_wolf", c, Vector2{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==10),
        };
        loot_wolf.children.push(GoalConnection{
            child: sell.clone(),
            is_additive: true,
            amplifier: 12.0,
        });
        attack_wolf.children.push(GoalConnection{
            child: Arc::new(loot_wolf),
            is_additive: false,
            amplifier: 1.0,
        });
        find_wolf.children.push(GoalConnection{
            child: Arc::new(attack_wolf),
            is_additive: false,
            amplifier: 1.0,
        });
        hunt.children.push(GoalConnection{
            child: Arc::new(find_wolf),
            is_additive: false,
            amplifier: 1.0,
        });

        root.children.push(GoalConnection{
            child: Arc::new(gather),
            is_additive: false,
            amplifier: 1.0,
        });
        root.children.push(GoalConnection{
            child: Arc::new(hunt),
            is_additive: false,
            amplifier: 1.0,
        });

        root
    }
    

    #[test]
    fn reality_exists() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    #[should_panic]
    fn how_to_rc_refcell() {
        let r = Rc::new(RefCell::new(Vector2{x: 0, y:0}));
        let mut r2 = r.deref().borrow_mut();
        r2.x = 5;
        let mut d = r.deref().borrow_mut();
        d.x = 6;
        r2.x = 10;
        //assert_eq!(r.clone().deref().borrow_mut().x, 10);
    }

    #[test]
    fn how_mut_ref_works() {
        fn my_mut(loc: &mut Vector2) {
            loc.x +=1;
            if loc.x < 10 {
                my_mut(loc);
            }
            loc.x +=1;
        }
        let mut loc = Vector2{x:0, y:0};
        my_mut(&mut loc);
        loc.x -= 5;
        my_mut(&mut loc);
        loc.y += 1;
    }

    #[test]
    fn how_vecs_ownership_works() { 
        let mut vec1 = vec![MapState::default()];
        let mut vec2 :Vec<MapState> = Vec::new();
        let trans = vec1.remove(0);
        vec2.push(trans);
        assert_eq!(vec1.len() + 1, vec2.len());
    }

    #[test]
    fn how_does_mut_ref_work() {
        fn need_immutable(loc: &Vector2) -> i32 {
            loc.x
        }
        fn need_mutable(loc: &mut Vector2) -> i32 {
            loc.x += 1;
            loc.x
        }

        let mut loc = Vector2{x: 1, y:2};
        let loc_m = &mut loc;
        need_immutable(loc_m);
        need_mutable(loc_m);
        need_immutable(loc_m);
        need_mutable(loc_m);
        assert_eq!(loc.x, 3);
    }

    #[test]
    fn how_does_mut_state_work_nested_obj() {
        struct MutMl<'a> {
            ml: &'a mut MapLocation,
        }

        fn use_ml(ml: &MapLocation) -> i32 {
            ml.location.x
        }
        fn change_ml(ml: &mut Vector2) {
            ml.x += 1;
        }

        let mut ml = MapLocation::default();

        let mml = MutMl{
            ml: &mut ml,
        };
        // both of below won't work!
        
        // let mml2 = MutMl{
        //     ml: &mut ml,
        // };
        //use_ml(&ml);
    }

    // should be
    // loc x=1, y=0 -> berry wins
    #[test]
    fn berry_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 1, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "berry"),
            _ => panic!("should return moveto!"),
        };
    }
    // loc x=1 y=1 -> fruit wins
    #[test]
    fn fruit_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 1, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "fruit"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=0 y=0 -> hunt deer wins
    #[test]
    fn find_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 0, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "find_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=5 -> attack deer
    #[test]
    fn attack_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 5, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "attack_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=6 -> loot deer
    #[test]
    fn loot_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 6, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "loot_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=7 y=0 -> eat deer (req met for eat if x==7 and y==0)
    #[test]
    fn eat_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 7, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "eat"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=7 y=1 -> sell deer (req met for sell if x==7 and y==1) OR x==11 (sell wolf)
    #[test]
    fn sell_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 7, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "sell"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=9 -> attack wolf
    #[test]
    fn attack_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 9, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "attack_wolf"),
            _ => panic!("should return moveto!"),
        };
    }

    // x=10 -> loot wolf
    #[test]
    fn loot_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 10, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "loot_wolf"),
            _ => panic!("should return moveto!"),
        };
    }

    // x=11 -> sell wolf
    #[test]
    fn sell_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Vector2{x: 11, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "sell"),
            _ => panic!("should return moveto!"),
        };
    }

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
    fn how_does_lifetime_loops() {
        let mut v = vec![1,2,3];
        fn fun (vv: &mut Vec<i32>) {
            vv[0] += 1;
        }
        for _ in 0..3 {
            let f = &mut v;
            fun(f);
        }
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
                        location: Vector2{x, y},
                        creatures: Some(Vec::new()),
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
                location: Vector2{x: 1, y: 1}
            };
        
            let mut deer2 =CreatureState{
                components: ComponentMap::default(),
                inventory: Vec::new(),
                memory: CreatureMemory::default(),
            };
            deer2.components.location_component = LocationComponent {
                location: Vector2{x: 1, y: 1}
            };
            let deer1_id = deer1.components.id_component.id;
            let deer2_id = deer2.components.id_component.id;
            region.grid[1][1].creatures.as_mut().unwrap().push(
                deer1
            );
            region.grid[1][1].creatures.as_mut().unwrap().push(
                deer2
            );
            region.grid[1][1].items.push(Item{
                item_type: ItemType::Berry,
                quantity: 1,
            });
            let berry_id = region.grid[1][1].id_component_items.id;

            let loc = &mut region.grid[1][1];
            let mut iter_mut = loc.creatures.as_mut().unwrap().iter_mut();
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
            let total: u32 = region.grid[1][1].creatures.as_mut().unwrap().iter().map(|c| {
                let ret: u32 = c.inventory.iter().map(|i| i.quantity).sum();
                ret
            }).sum();
            assert_eq!(total, 1);
            1
        }).sum();
        assert_eq!(y, 100);
    }
    
}
