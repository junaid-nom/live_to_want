

use std::cmp::Ordering;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fmt::{Debug, Formatter}, borrow::Borrow};
use std::sync::atomic::AtomicU64;
use core::fmt;

extern crate rayon;
use rayon::prelude::*;

// NOTE: All event chains with items need to end in a final failure case of putting item on ground.
// this is because you can try to give an item away as someone else fills your inventory and 
// if both giving fails and putting back into your inventory fails, need to put item somewhere, so put on ground.

static COUNTER: AtomicU64 = AtomicU64::new(1); // TODO: Upgrade to a 128 bit one when it comes out of nightly build
fn get_id() -> u64 { COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }
type UID = u64;

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
    location_component: Option<LocationComponent>,
    region_component: Option<RegionComponent>,
    name_component: Option<NameComponent>,
    creature_type_component: Option<CreatureTypeComponent>,
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
}
impl Component for HealthComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct LocationComponent {
    location: Location,
}
impl Component for LocationComponent {
    fn get_visible() -> bool {
        true
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct RegionComponent {
    region: Location,
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

// pub enum CreatureComponents {
//     Health(i32),
//     Location(Location),
//     Region(Location),
//     Name(String),
//     CreatureType(CreatureType),
// }

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq)]
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
    fn new<'a>(loc: Location) -> CreatureState {
        let mut ret = CreatureState::default();
        ret.components.location_component = Some(LocationComponent{location:loc});
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
pub struct Location {
    x: i32,
    y: i32,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapState {
    regions: Vec<Vec<MapRegion>>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion {
    grid: Vec<Vec<MapLocation>>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation {
    id_component: IDComponent, 
    location: Location,
    creatures: Vec<Rc<RefCell<CreatureState>>>,
    items: Rc<RefCell<Vec<Item>>>,
}

#[derive(Debug)]
pub enum CreatureCommand<'a>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'a str, Location),
    Chase(&'a str, &'a ComponentMap),
    Attack(&'a str, &'a ComponentMap),
}

pub struct GoalConnection<'a> {
    pub child: Rc<GoalNode<'a>>,
    pub is_additive: bool,
    pub amplifier: f32,
}

/// Is a list of all events for that target for a given frame cycle
/// Must place all tasks for that target in here at once or could cause race conditions
//#[derive(std::marker::Sized)] doesnt work...
pub struct TaskList {
    target: EventTarget,
    tasks: Vec<EventChain>,
}
impl TaskList {
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
        let success = e.get_requirements.deref()(&*effected);
        if success {
            e.mutate(effected);
            let mut se = self;
            se.index+=1;
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
#[derive(PartialEq, Eq, Clone)]
pub enum EventTarget {
    LocationItemTarget(Rc<RefCell<Vec<Item>>>, UID),
    CreatureTarget(Rc<RefCell<CreatureState>>),
}
impl EventTarget {
    fn get_id(&self) -> UID {
        match &self {
            EventTarget::LocationItemTarget(_, id) => {*id}
            EventTarget::CreatureTarget(c) => {c.deref().borrow().components.id_component.id}
        }
    }
}

pub struct Event {
    event_type: EventType,
    get_requirements: Box<dyn Fn (&EventTarget) -> bool>,
    on_fail: Option<EventChain>,
    target: EventTarget,
}
impl Event {
    fn mutate(&self, effected: &mut EventTarget) {
        match &self.event_type {
            EventType::Move(_) => {}
            EventType::RemoveItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        for v in v.deref().borrow_mut().iter_mut() {
                            if v.item_type == *t {
                                v.quantity -= q;
                                return;
                            }
                        }
                    }
                    EventTarget::CreatureTarget(c) => {
                        for v in c.deref().borrow_mut().inventory.iter_mut() {
                            if v.item_type == *t {
                                v.quantity -= q;
                                return;
                            }
                        }
                    }
                }
                panic!(format!("Failed to find item in event! event: {:#?}", &self));
            }
            EventType::AddItem(q, t) => {
                match effected {
                    EventTarget::LocationItemTarget(v, _) => {
                        let mut inventory = v.deref().borrow_mut();
                        for v in inventory.iter_mut() {
                            if v.item_type == *t {
                                v.quantity += q;
                                return;
                            }
                        }
                        inventory.push(Item{
                            item_type: *t,
                            quantity: *q,
                        });
                    }
                    EventTarget::CreatureTarget(c) => {
                        let mut c = c.deref().borrow_mut();
                        for v in c.inventory.iter_mut() {
                            if v.item_type == *t {
                                v.quantity += q;
                                return;
                            }
                        }
                        c.inventory.push(Item{
                            item_type: *t,
                            quantity: *q,
                        });
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
    Move(Location),
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
                // if true {
                //     let cref: Rc<RefCell<GoalCacheNode<'a>>> = existing_caches.get(child_goal.child.name).unwrap().clone();
                //     GoalCacheNode::setup_children( cref.clone(), map_state, c_state, existing_caches);
                //     if let Some(children) = &mut goal_cache.children {
                //         children.push(cref.clone());
                //     }
                // } else {
                //     let new_child = ;
                //     let name: &'a str = new_child.deref().borrow().goal.name;
                //     existing_caches.insert(name, new_child.clone());
                //     GoalCacheNode::setup_children(new_child.clone(), map_state, c_state, existing_caches);
                // };
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


pub struct GoalNode<'a> {
    get_want_local: Box<dyn Fn(&MapState, &CreatureState) -> u32>,
    get_effort_local: Box<dyn Fn(&MapState, &CreatureState) -> u32>,
    children: Vec<GoalConnection<'a>>,
    name: &'a str,  // just for debugging really
    get_command: Option<Box<dyn for<'f> Fn(&MapState, &'f CreatureState) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of a organizing node
    get_requirements_met: Box<dyn Fn (&MapState, &CreatureState) -> bool>,
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
                if c.components.location_component.as_ref().unwrap().location.x == 1 {
                    30
                } else {
                    50
                }
            }),
            children: Vec::new(),
            name: "berry",
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("berry", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        let fruit = GoalNode {
            get_want_local: Box::new(|_, c| {
                if c.components.location_component.as_ref().unwrap().location.y == 1 {
                    101
                } else {
                    99
                }
            }),
            get_effort_local: Box::new(|_, c| {
                if c.components.location_component.as_ref().unwrap().location.x == 1 {
                    30
                } else {
                    50
                }
            }),
            children: Vec::new(),
            name: "fruit",
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("fruit", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, _| true),
        };
        gather.children.push(GoalConnection{
            child: Rc::new(berry),
            is_additive: false,
            amplifier: 1.0,
        });
        gather.children.push(GoalConnection{
            child: Rc::new(fruit),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("find_deer", Location{x: 0, y:0}))),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("attack_deer", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.x==5),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("loot_deer", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.x==6),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("eat", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.y==0 && c.components.location_component.as_ref().unwrap().location.x==7),
        };
        let eat = Rc::new(eat);
        let sell = GoalNode {
            get_want_local: Box::new(|_, _| {
                10
            }),
            get_effort_local: Box::new(|_, _| {
                1
            }),
            children: Vec::new(),
            name: "sell",
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("sell", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.y==1 && 
                (c.components.location_component.as_ref().unwrap().location.x==7 || c.components.location_component.as_ref().unwrap().location.x==11)),
        };
        let sell = Rc::new(sell);

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
            child: Rc::new(loot_deer),
            is_additive: false,
            amplifier: 1.0,
        });
        find_deer.children.push(GoalConnection{
            child: Rc::new(attack_deer),
            is_additive: false,
            amplifier: 1.0,
        });
        hunt.children.push(GoalConnection{
            child: Rc::new(find_deer),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("find_wolf", Location{x: 0, y:0}))),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("attack_wolf", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.x==9),
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
            get_command: Some(Box::new(|_, _| CreatureCommand::MoveTo("loot_wolf", Location{x: 0, y:0}))),
            get_requirements_met: Box::new(|_, c| c.components.location_component.as_ref().unwrap().location.x==10),
        };
        loot_wolf.children.push(GoalConnection{
            child: sell.clone(),
            is_additive: true,
            amplifier: 12.0,
        });
        attack_wolf.children.push(GoalConnection{
            child: Rc::new(loot_wolf),
            is_additive: false,
            amplifier: 1.0,
        });
        find_wolf.children.push(GoalConnection{
            child: Rc::new(attack_wolf),
            is_additive: false,
            amplifier: 1.0,
        });
        hunt.children.push(GoalConnection{
            child: Rc::new(find_wolf),
            is_additive: false,
            amplifier: 1.0,
        });

        root.children.push(GoalConnection{
            child: Rc::new(gather),
            is_additive: false,
            amplifier: 1.0,
        });
        root.children.push(GoalConnection{
            child: Rc::new(hunt),
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
        let r = Rc::new(RefCell::new(Location{x: 0, y:0}));
        let mut r2 = r.deref().borrow_mut();
        r2.x = 5;
        let mut d = r.deref().borrow_mut();
        d.x = 6;
        r2.x = 10;
        //assert_eq!(r.clone().deref().borrow_mut().x, 10);
    }

    #[test]
    fn how_mut_ref_works() {
        fn my_mut(loc: &mut Location) {
            loc.x +=1;
            if loc.x < 10 {
                my_mut(loc);
            }
            loc.x +=1;
        }
        let mut loc = Location{x:0, y:0};
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
        fn need_immutable(loc: &Location) -> i32 {
            loc.x
        }
        fn need_mutable(loc: &mut Location) -> i32 {
            loc.x += 1;
            loc.x
        }

        let mut loc = Location{x: 1, y:2};
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
        fn change_ml(ml: &mut Location) {
            ml.x += 1;
        }

        let mut ml = MapLocation{
            location: Location{x: 0, y: 0},
            creatures: Vec::new(),
            items: Rc::new(RefCell::new(Vec::new())),
            id_component: IDComponent::new(),
        };

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
        let c_s = CreatureState::new(Location{x: 1, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "berry"),
            _ => panic!("should return moveto!"),
        };
    }
    // loc x=1 y=1 -> fruit wins
    #[test]
    fn fruit_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 1, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "fruit"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=0 y=0 -> hunt deer wins
    #[test]
    fn find_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 0, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "find_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=5 -> attack deer
    #[test]
    fn attack_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 5, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "attack_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=6 -> loot deer
    #[test]
    fn loot_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 6, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "loot_deer"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=7 y=0 -> eat deer (req met for eat if x==7 and y==0)
    #[test]
    fn eat_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 7, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "eat"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=7 y=1 -> sell deer (req met for sell if x==7 and y==1) OR x==11 (sell wolf)
    #[test]
    fn sell_deer_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 7, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "sell"),
            _ => panic!("should return moveto!"),
        };
    }
    // x=9 -> attack wolf
    #[test]
    fn attack_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 9, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "attack_wolf"),
            _ => panic!("should return moveto!"),
        };
    }

    // x=10 -> loot wolf
    #[test]
    fn loot_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 10, y:0});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "loot_wolf"),
            _ => panic!("should return moveto!"),
        };
    }

    // x=11 -> sell wolf
    #[test]
    fn sell_wolf_wins() {
        let root = generate_basic_graph();
        let m_s = MapState::default();
        let c_s = CreatureState::new(Location{x: 11, y:1});
        let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
        let res = res.unwrap();
        println!("Got: {:#?}", &res);

        match res {
            CreatureCommand::MoveTo(n, _) => assert_eq!(n, "sell"),
            _ => panic!("should return moveto!"),
        };
    }

    #[test]
    fn test_rayon() {
        pub struct TaskListTest {
            ev: EventTarget,
        }
        let ev = vec!(EventTarget::LocationItemTarget(Rc::new(RefCell::new(Vec::new())), 1));
        let ev = vec![Rc::new(RefCell::new(2))];
        let ev = vec![2];
        ev.into_par_iter().map(|x| match x {
            EventTarget::LocationItemTarget(_, _) => {}
            EventTarget::CreatureTarget(_) => {}
        });
    }

    #[test]
    fn test_chain_multithread() {
        // make a mapstate with some deer
        let mut region = MapRegion{
            grid:Vec::new()
        };
        for x in 0..10 {
            let mut xList  = Vec::new();
            for y in 0..10 {
                let loc = MapLocation{
                    id_component: IDComponent::new(),
                    location: Location{x, y},
                    creatures: Vec::new(),
                    items: Rc::new(RefCell::new(Vec::new())),
                };
                xList.push(loc);
            }
            region.grid.push(xList);
        }
    
        let mut deer1 = Rc::new(RefCell::new(CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        }));
        deer1.deref().borrow_mut().components.location_component.as_mut().unwrap().location.x = 1;
        deer1.deref().borrow_mut().components.location_component.as_mut().unwrap().location.y = 1;
    
        region.grid[1][1].creatures.push(
            deer1.clone()
        );
    
        let mut deer2 = Rc::new(RefCell::new(CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        }));
        deer2.deref().borrow_mut().components.location_component.as_mut().unwrap().location.x = 1;
        deer2.deref().borrow_mut().components.location_component.as_mut().unwrap().location.y = 1;
    
        region.grid[1][1].creatures.push(
            deer2.clone()
        );
    
        region.grid[1][1].items.deref().borrow_mut().push(Item{
            item_type: ItemType::Berry,
            quantity: 1,
        });
        
        // make some event chain examples
        // pick up item -> remove item (if fail remove item again) (note, in rl would do reverse)
        let pickup1 = Event {
            event_type: EventType::AddItem(1, ItemType::Berry),
            get_requirements: Box::new(|_| true),
            on_fail: None,
            target: EventTarget::CreatureTarget(deer1.clone()),
        };
        let pickup2 = Event {
            event_type: EventType::AddItem(1, ItemType::Berry),
            get_requirements: Box::new(|_| true),
            on_fail: None,
            target: EventTarget::CreatureTarget(deer2.clone()),
        };
        let pickup_fail = Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|_| true),
            on_fail: None,
            target: EventTarget::CreatureTarget(deer1.clone()),
        };
        let event_fail1 = EventChain {
            index: 0,
            events: vec!(pickup_fail),
        };
        let pickup_fail2 = Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|_| true),
            on_fail: None,
            target: EventTarget::CreatureTarget(deer2.clone()),
        };
        let event_fail2 = EventChain {
            index: 0,
            events: vec!(pickup_fail2),
        };
        let remove1=  Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|e| {
                match e {
                    EventTarget::LocationItemTarget(i, _) => {
                        for item in i.deref().borrow().iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    EventTarget::CreatureTarget(c) => {
                        for item in c.deref().borrow().inventory.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                }
            }),
            on_fail: Some(event_fail1),
            target: EventTarget::LocationItemTarget(region.grid[1][1].items.clone(), region.grid[1][1].id_component.id)
        };
        let remove2=  Event {
            event_type: EventType::RemoveItem(1, ItemType::Berry),
            get_requirements: Box::new(|e| {
                match e {
                    EventTarget::LocationItemTarget(i, _) => {
                        for item in i.deref().borrow().iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                    EventTarget::CreatureTarget(c) => {
                        for item in c.deref().borrow().inventory.iter() {
                            if item.item_type == ItemType::Berry && item.quantity > 0 {
                                return true
                            }
                        }
                        false
                    }
                }
            }),
            on_fail: Some(event_fail2),
            target: EventTarget::LocationItemTarget(region.grid[1][1].items.clone(), region.grid[1][1].id_component.id)
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
    
        let mut tasks_map: HashMap<UID, TaskList> = HashMap:: new();
        for ec in event_chains.into_iter() {
            let key = ec.events[ec.index].target.get_id();
            match tasks_map.get_mut(&key) {
                Some(tl) => {
                    tl.tasks.push(ec);
                }
                None => {
                    let tl = TaskList {
                        target: ec.events[ec.index].target.clone(),
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
        //let mut test_rayon = vec![1,2,3,4,5];
        //test_rayon.into_par_iter().map(|tl| (tl*2));
    
    
        let next = task_lists.into_par_iter().flat_map(move |tl| tl.process());
    
    }
}
