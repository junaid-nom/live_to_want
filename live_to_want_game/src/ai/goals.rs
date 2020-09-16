use crate::{map_state::MapState, creature::CreatureState, map_state::CreatureCommand};
use std::{cell::RefCell, rc::Rc, sync::Arc, collections::HashMap};
use std::ops::Deref;
use std::borrow::Borrow;
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
    pub get_want_local: Box<fn(&MapState, &CreatureState) -> u32>,
    pub get_effort_local: Box<fn(&MapState, &CreatureState) -> u32>,
    pub children: Vec<GoalConnection<'a>>,
    pub name: &'a str,  // just for debugging really
    pub get_command: Option<Box<for<'f, 'c> fn(&MapState, &'f CreatureState) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of a organizing node
    pub get_requirements_met: Box<fn (&MapState, &CreatureState) -> bool>,
}

impl GoalNode<'_> {
}

pub struct GoalConnection<'a> {
    pub child: Arc<GoalNode<'a>>,
    pub is_additive: bool,
    pub amplifier: f32,
}

pub struct GoalCacheNode<'a> {
    pub goal: &'a GoalNode<'a>,
    pub children: Option<Vec<Rc<RefCell<GoalCacheNode<'a>>>>>,
    pub want_local: u32,
    pub effort_local: u32,
    pub requirement_met: bool,
    pub motivation_global: Option<f32>, // if None is not calculated yet. Should be (sum(want/effort for all children) + local want) / local effort
}
impl GoalCacheNode<'_> {
    pub fn new<'a>(goal: &'a GoalNode, map_state :&MapState, c_state : &CreatureState) -> GoalCacheNode<'a> {
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

    pub fn _my_func(num: i32, list_of_nums: &mut Vec<i32>) {
        list_of_nums.push(num);
        if num - 1 >= 0 {
            GoalCacheNode::_my_func(num - 1, list_of_nums);
        }
    }

    pub fn _my_fc<'a>() -> Option<MapState> {
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

    pub fn setup_children<'a>(goal_cache:  Rc<RefCell<GoalCacheNode<'a>>>, map_state :&MapState, c_state : &CreatureState, existing_caches: Rc<RefCell<HashMap<&'a str, Rc<RefCell<GoalCacheNode<'a>>>>>>) {
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

    pub fn get_connection_by_name<'a>(goal_parent: &'a GoalNode, child_name: &str) -> Option<&'a GoalConnection<'a>> {
        for c in &goal_parent.children {
            if c.child.deref().name == child_name {
                return Some(c);
            }
        }
        None
    }

    //note must call setup_children first
    pub fn setup_global_stats(goal_cache:  Rc<RefCell<GoalCacheNode>>, map_state :&MapState, c_state : &CreatureState) {
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

pub fn generate_goal_nodes<'a>() -> GoalNode<'a> {
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
