

use std::cmp::Ordering;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::borrow::Borrow;

#[derive(Debug)]
pub enum CreatureType {
    Deer,
    Wolf,
    Human,
    Tree,
}
impl Default for CreatureType {
    fn default() -> Self { CreatureType::Deer }
}

#[derive(Debug)]
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
#[derive(Default)]
pub struct CreatureAttributes {

}

#[derive(Debug)]
#[derive(Default)]
pub struct Item {
    pub item_type: ItemType,
    pub quantity: u32,
}

#[derive(Debug)]
#[derive(Default)]
pub struct CreatureState<'a> {
    pub attributes: CreatureAttributes,
    pub memory: CreatureMemory,
    pub visible_state: CreatureVisibleState<'a>,
    pub inventory: Vec<Box<Item>>,
}
impl CreatureState<'_> {
    fn new<'a>(loc: Location) -> CreatureState<'a> {
        let mut ret = CreatureState::default();
        ret.visible_state.location = loc;
        ret
    }
}

#[derive(Debug)]
#[derive(Default)]
pub struct CreatureVisibleState<'a> {
    pub location: Location,
    pub region: Location,
    pub name: &'a str,
    pub creature_type: CreatureType,
}

#[derive(Debug)]
#[derive(Default)]
pub struct CreatureMemory {
    
}

#[derive(Debug)]
#[derive(Default)]
#[derive(Copy, Clone)]
pub struct Location {
    x: i32,
    y: i32,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapState<'a> {
    regions: Vec<Vec<MapRegion<'a>>>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapRegion<'a> {
    grid: Vec<Vec<MapLocation<'a>>>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct MapLocation<'a> {
    location: Location,
    creatures: Vec<CreatureState<'a>>,
    items: Vec<Item>,
}

#[derive(Debug)]
pub enum CreatureCommand<'a>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'a str, Location),
    Chase(&'a str, &'a CreatureVisibleState<'a>),
    Attack(&'a str, &'a CreatureVisibleState<'a>),
}

pub struct GoalConnection<'a> {
    pub child: Rc<GoalNode<'a>>,
    pub is_additive: bool,
    pub amplifier: f32,
}

pub struct TaskList {

}
pub struct EventChain {
    
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

    fn _my_fc<'a>() -> Option<MapState<'a>> {
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
    // fn get_total_motivation(&mut self, map_state :&MapState, c_state : &CreatureState) -> u32 {
    //     if let Some(cached_motivation) = self.cached_motivation {
    //         return cached_motivation;
    //     } else {
    //         let mut max = 0;
    //         for c in &mut self.children {
    //             let c_mot = c.get_total_motivation(map_state, c_state);
    //             if c_mot > max {
    //                 max = c_mot;
    //             }
    //         }
    //         self.cached_motivation = Some(max);
    //         max
    //     }
    // }

    // fn get_final_node<'a>(&'a mut self, map_state :&MapState, c_state : &CreatureState) -> &'a GoalNode {
    //     // let max_node = self.children.iter_mut().max_by(|a, b| {
    //     //     let a_mot = a.get_total_motivation(map_state, c_state);
    //     //     let b_mot = b.get_total_motivation(map_state, c_state);
    //     //     a_mot.cmp(&b_mot)
    //     // });

    //     let mut max = 0;
    //     let mut max_node = None;
    //     for c in &mut self.children {
    //         let c_mot = c.get_total_motivation(map_state, c_state);
    //         if c_mot > max {
    //             max = c_mot;
    //             max_node = Some(c);
    //         }
    //     }

    //     if let Some(max_n) = max_node {
    //         max_n
    //     } else {
    //         &self
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{cell::{RefCell}, rc::Rc};
    //use std::collections::HashMap;

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
                if c.visible_state.location.x == 1 {
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
                if c.visible_state.location.y == 1 {
                    101
                } else {
                    99
                }
            }),
            get_effort_local: Box::new(|_, c| {
                if c.visible_state.location.x == 1 {
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.x==5),
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.x==6),
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.y==0 && c.visible_state.location.x==7),
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.y==1 && 
                (c.visible_state.location.x==7 || c.visible_state.location.x==11)),
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.x==9),
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
            get_requirements_met: Box::new(|_, c| c.visible_state.location.x==10),
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
}
