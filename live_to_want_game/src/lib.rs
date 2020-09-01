#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::cmp::Ordering;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::borrow::Borrow;

pub struct CreatureAttributes {

}
pub struct CreatureState<'a> {
    attributes: CreatureAttributes,
    memory: CreatureMemory,
    visible_state: CreatureVisibleState<'a>,
}
pub struct CreatureVisibleState<'a> {
    location: &'a Location,
}

pub struct CreatureMemory {
    
}

pub struct Location {
    x: i32,
    y: i32,
}

pub struct MapState {

}
pub enum CreatureCommand<'a>{
    MoveTo(&'a Location),
    Chase(&'a CreatureVisibleState<'a>),
    Attack(&'a CreatureVisibleState<'a>),
}

pub struct GoalConnection<'a> {
    child: &'a GoalNode<'a>,
    is_additive: bool,
    amplifier: f32,
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

    fn my_func(num: i32, list_of_nums: &mut Vec<i32>) {
        list_of_nums.push(num);
        if num - 1 >= 0 {
            GoalCacheNode::my_func(num - 1, list_of_nums);
        }
    }

    fn my_fc() -> Option<MapState> {
        let poop : Option<MapState>;
        poop = Some(MapState{});
        // MUST USE & IN FRONT OF OPTION SO IT DOESNT GET TAKEN!
        let p: Option<MapState> = match &poop {
            Some(n) => None,
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
                let mut existing_cache_ref = existing_caches.borrow_mut();
                let entry = existing_cache_ref.entry(child_goal.child.name).or_insert(
        {
                    Rc::new(RefCell::new(GoalCacheNode::new(child_goal.child, map_state, c_state)))
                });
                let cref: Rc<RefCell<GoalCacheNode<'a>>> = entry.clone();
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

    //note must call setup_children first
    fn setup_global_stats(goal_cache:  Rc<RefCell<GoalCacheNode>>, map_state :&MapState, c_state : &CreatureState) {
        let goal_cache = goal_cache.clone();
        let mut goal_cache = goal_cache.borrow_mut();
        if let Some(_) = goal_cache.motivation_global {
            return
        } else {
            let mut sum_motivation: f32 = goal_cache.want_local as f32;
            if let Some(children) = &goal_cache.children {
                for c in children {
                    let c_ref = c.clone();
                    
                    if let None = c_ref.borrow_mut().motivation_global {
                        GoalCacheNode::setup_global_stats(c_ref.clone(), map_state, c_state);
                    }

                    sum_motivation += c_ref.borrow_mut().motivation_global.unwrap();
                    sum_motivation = sum_motivation / (goal_cache.effort_local as f32);
                }
            }
            goal_cache.motivation_global = Some(sum_motivation);
        }
    }

    fn get_final_command<'a, 'b>(goal_node: &'a GoalNode, map_state :&MapState, c_state : &'b CreatureState) -> Option<CreatureCommand<'b>> { 
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
            let req_met = (look_at.goal.get_requirements_met)(map_state, c_state);

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
