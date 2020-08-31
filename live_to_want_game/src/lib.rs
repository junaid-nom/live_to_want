#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::cmp::Ordering;
use std::{cell::RefCell, rc::Rc};
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

    fn setup_children(goal_cache: &mut GoalCacheNode, map_state :&MapState, c_state : &CreatureState, existing_caches: &mut HashMap<&str, Rc<RefCell<GoalCacheNode>>>) {
        if let Some(_) = goal_cache.children {
            // this node is setup already
            return;
        } else {
            goal_cache.children = Some(Vec::new());
            for child_goal in &goal_cache.goal.children {
                if existing_caches.contains_key(child_goal.child.name) {
                    let cref = existing_caches.get(child_goal.child.name).as_mut().unwrap();
                    GoalCacheNode::setup_children(cref.get_mut(), map_state, c_state, existing_caches);
                    goal_cache.children.unwrap().push((*cref).clone());
                } else {
                    let mut new_child = Rc::new(RefCell::new(GoalCacheNode::new(child_goal.child, map_state, c_state)));
                    let name = new_child.clone().get_mut().goal.name;
                    existing_caches.insert(name, new_child.clone());
                    GoalCacheNode::setup_children(new_child.get_mut(), map_state, c_state, existing_caches);
                };
            }
        }
    }

    //note must call setup_children first
    fn setup_global_stats(goal_cache: &mut GoalCacheNode, map_state :&MapState, c_state : &CreatureState) {
        if let Some(_) = goal_cache.motivation_global {
            return
        } else {
            let mut sum_motivation: f32 = goal_cache.want_local as f32;
            for c in &mut goal_cache.children.unwrap() {
                let c_ref = c.get_mut();
                if let None = c_ref.motivation_global {
                    GoalCacheNode::setup_global_stats(c_ref, map_state, c_state);
                }
                sum_motivation += c_ref.motivation_global.unwrap();
                sum_motivation = sum_motivation / (goal_cache.effort_local as f32);
            }
            goal_cache.motivation_global = Some(sum_motivation);
        }
    }

    fn get_final_command<'a, 'b>(goal_node: &'a GoalNode, map_state :&MapState, c_state : &'b CreatureState) -> Option<CreatureCommand<'b>> { 
        let mut parent = GoalCacheNode::new(goal_node, map_state, c_state);
        let mut existing_caches: HashMap<&str, Rc<RefCell<GoalCacheNode>>> = HashMap::new();
        GoalCacheNode::setup_children(&mut parent, map_state, c_state, &mut existing_caches);
        GoalCacheNode::setup_global_stats(&mut parent, map_state, c_state);

        // now go through the tree. if requirements met, go into it, if not ignore it. Find best
        // Node. then run the command function on that node.
        let mut to_visit : Vec<&GoalCacheNode>= Vec::new();
        let mut visited : usize = 0;
        to_visit.push(&parent);
        let mut best_node : Option<&GoalCacheNode> = None;

        while to_visit.len() - visited > 0 {
            visited+=1;
            let look_at: &GoalCacheNode = to_visit[visited];
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
                        if look_at.motivation_global >= n.motivation_global {
                            best_node = Some(look_at);
                        }
                    },
                    None => {
                        best_node = Some(look_at);
                    }
                }
            }
            
            for c in & look_at.children.unwrap() {
                let c_ref = (*c).clone().get_mut();

                if !(to_visit.iter().any(|c| c.goal.name == c_ref.goal.name)) {
                    to_visit.push((*c).clone().get_mut());
                }
            }
        }

        match best_node {
            Some(n) => Some((n.goal.get_command).unwrap()(map_state, c_state)),
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
