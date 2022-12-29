use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::{UID, MapState, CreatureState, CreatureCommand, Location};

pub type NodeIndex = usize;

pub fn get_count_of_variable(m: &MapState, c: &CreatureState, v: Variable) -> i32 {
    // TODO get the count of each variable.
    // Most will be what does creature have in inventory.
    // others could be result of a function for example "whats my power level"
    // could even be something like "my rank in power compared to creatures near me"
    match v {
        _ => {
            // TODO
        },
    }

    0
}

pub fn get_global_reward_for_connection(node: &Node, global_reward: f32, base_multiplier: f32, count: i32, m: &MapState, c: &CreatureState, c_target: Option<&CreatureState>) -> f32 {
    match node {
        Node::Reward(n) => n.reward_connection.as_ref()(m, c, count) * global_reward,
        Node::CreatureList(nl) => nl.reward_connection.as_ref()(m, c, count, c_target.unwrap()) * global_reward,
    }
}

pub fn get_variable_change_from_vec_vec(changes :&Vec<Vec<VariableChange>>, variable: Variable) -> Option<VariableChange> {
    for vc_list in changes {
        for vc in vc_list {
            if vc.variable == variable {
                return Some(*vc);
            }
        }
    }
    None
}

pub fn get_variable_change_from_effects(category: Variable, effects: &Vec<VariableChange>) -> Option<VariableChange> {
    for vc in effects.iter() {
        if vc.variable == category {
            return Some(*vc);
        }
    }

    None
}

pub struct RootNode {
    pub description: String,  // just for debugging/comments
    pub nodes: Vec<Node>,
    pub children: Vec<RewardNodeConnection>,
}
impl RootNode {
    pub fn generate_result_graph(&self, map_state :&MapState, c_state : &CreatureState) -> NodeResultRoot {
        let mut root = NodeResultRoot{
            children: vec![],
            original_node_descriptor: self.description.clone(),
            nodes: vec![],
        };
        for node in &self.nodes {
            match node{
                Node::Reward(n) => {
                    let requirement = n.requirement.as_ref()(map_state, c_state);
                    let reward = n.reward.as_ref()(map_state, c_state, &requirement);
                    let cost = n.cost.as_ref()(map_state, c_state, &requirement);
                    let effects = match &n.effect {
                        Some(e) => e.as_ref()(map_state, c_state, &reward, &requirement),
                        None => vec![],
                    };

                    let new = NodeResult{
                        original_node_description: n.description.clone(), // NOTE: Is this worth? Every frame copying over the description. All to make debug print easier?
                        requirement_result: requirement,
                        reward_result: reward,
                        cost_result: cost,
                        global_reward: NodeRewardGlobal { rewards_per_requirement: vec![], reward_sum_total: None, reward_global_with_costs: None },
                        children: (&n.children).into_iter().map(|c| c.child_index).collect(),
                        effects: effects,
                        connection_results: None, // need to wait for global results of children to compute this
                        original_node: n.index,
                        creature_target: None,
                    };
                    root.nodes.push(new);
                },
                Node::CreatureList(nl) => todo!(),
            }
        }


        // Now how to get the reward of everything...
        // I guess: go through from root, if node doesn't have global reward set yet, then, calculate the global reward on it.
        // First step of that is to calculate it on its children recurssively
        for i in 0..root.children.len() {
            let mut indexs_processed = HashSet::new();
            // TODO: Converting the UID of creature target, to the creature state itself
            // is gonna be REALLY confusing. Should probably just be a reference to 
            // a COPY of the CreatureState in the MEMORY of the Creature doing this AI.
            // but then will need to update the memory for all creatures currently in view... wtf?
            // ok so can't do that copy idea.
            // instead need to make a new helper function in map_state that can take a UID
            // and output a creature State (option) or whatever.
            // creature memory just stores UID and last location seen? only of important friendly stuff?
            // might be able to make a dictionary of UID->CreatureState within mapstate itself and save it? or maybe its just input to this function.
            root.calculate_global_reward( &self, map_state, c_state, None, i, &mut indexs_processed);
        }

        for node in &self.nodes {
            match node{
                Node::Reward(n) => {
                    let node_result = &mut root.nodes[n.index];

                    // If there is an inbetween material to make something, and we have enough to make it already.
                    // then we should add 1 to the actual count of the child object, because we already have enough to make one more of it for this ingredient.
                    // For example if it takes 5 wood to make a plank, and 1 plank and 1 stone to make a spear,
                    // then if you already have 5 wood, the reward of gathering more wood should be as if we already 
                    // have +1 spear than we have because we already have wood to make +1 spear.
                    let mut this_counts: Vec<i32> = node_result.effects.iter().map(|e| get_count_of_variable(map_state, c_state, e.variable)).collect();

                    // Need to do custom bonus_counts for EACH child count.
                    // get current amount of this node by checking its effects.
                    // so if the effects are wood and fiber. and we already have 5 of each.
                    // if three different children needs 1 wood, another two need 2 fiber, then
                    // we need to do the Limit algorithm. Where we see what the value of 
                    // adding another wood is for example, but first we need to see how much "wood" we 
                    // would allocate to each connection already.

                    // Wait a minute, every variable change should have its own node.
                    // and so nodes that have effects: +1 wood +1 fiber, would simply connect
                    // to that node. SO WE SHOULD NEVER HAVE A NODE THAT HAS MULTIPLE woods, AND multiple some other.
                    // So Count should always be a single i32.
                    // So chopTree-> (Wood->Many wood items), (Fiber-> many fiber items),
                    // so chopTree only has 1 item per category. Wood has many items for wood category but 
                    // only 1 category.

                    // below must be done AFTER all global rewards of children have been calculated!
                    // first, separate all connections into separate lists based on their Variable.
                    // we get the variable count for that category
                    // we also make a list of i32 of bonuses given to each child conn. starting at 0
                    // now, 0..VariableCount we get the reward for each connection for each VariableCount, 
                    // this is global_reward of child * child_reward_multiplier(count) * base_multiplier
                    // whatever the max is, increase its bonus_reward by 1.
                    // now do this one FINAL time which is for "we get 1 additional wood" and that is the final rewards
                    // for each connection, and the global reward for this will just be the Max of that.
                    // then sum up the Maxs for each category.
                    // that will then be used in the global reward.
                    // these partial results can be saved in the connection results for debugging.
                    let mut conn_by_categories: Vec<Vec<&RewardNodeConnection>> = vec![];
                    (&n.children).into_iter().for_each(|conn| {
                        let mut existing = false;
                        for cvec in conn_by_categories.iter_mut() {
                            if cvec.len() > 0 && cvec[0].requirement.variable == conn.requirement.variable {
                                existing = true;
                                cvec.push(conn);
                                break;
                            }
                        }
                        if !existing {
                            conn_by_categories.push(vec![conn]);
                        }
                    });

                    let mut conn_results: Vec<Vec<ConnectionResult>> = Vec::new();
                    // get actual counts of the variable change outputs of the child.
                    // so if a child of this node has: +2wood +3fiber. and we have 4 wood/fiber already
                    // and we have 
                    // reward should be per effect...
                    // so reward func should be -> Vec<VariableChange?, Reward?>

                    // to actually get hypothetical rewardd if more need recurse alot For every parent!!
                    // This would be way too expensive so instead just ESTIMATE with that connection multiplier func. Also to get super accurate count can take the local reward for that requirement/total reward and multiply that but the count for that requirement to get "how many of this node do we already have" and add that with "how many do we more because of how many of that requirement part we fulfilled."
                    // Still need to make all rewards split on per connection. Then can get a rough estimate of "count"
                    // by looking at each reward type and multiplying it by conn_reward/total_reward
                    let counts_default: Vec<i32> = vec![];
                    for conn_list in conn_by_categories.iter() {
                        let mut cat_results: Vec<ConnectionResult> = vec![];
                        for c in conn_list {
                            // cat_results.push(ConnectionResult{
                            //     multiplier: c.base_multiplier,
                            //     conn_reward: Some(
                            //         get_global_reward_for_connection(
                            //             (&self).nodes.get((*c).child_index).unwrap(), 
                            //             root.nodes[(*c).child_index].global_reward.unwrap(), 
                            //             &counts_default, 
                            //             map_state, 
                            //             c_state,
                            //             None)
                            //         ),
                            //     bonus_count: 0,
                            // });
                        }
                        let count = get_count_of_variable(map_state, c_state, conn_list[0].requirement.variable);
                        for i in 0..count {
                            // get the Max reward of them all, increase its count by 1 and recompute it.
                        }
                    }
                },
                Node::CreatureList(nl) => todo!(),
            }
        }

                    

        root
    }
}

pub enum Node {
    Reward(RewardNode),
    CreatureList(RewardNodeCreatureList),
}

#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum Variable {
    None,
    Bone,
    Skin,
    // NOTE inbetween ingredients will need to be variables. Anything that is an inner OR. For example, if  (wood OR clay) AND glue makes a wall, then (wood OR clay) must be its own node and variable.
}
#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct VariableChange {
    pub variable: Variable,
    pub change: i32,
}

pub struct RewardNodeConnection {
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex,
    pub requirement: VariableChange, // multiplier is: 1/requirement.change * effect(for that variable)
}
pub struct RewardNode {
    pub description: String,  // just for debugging/comments
    pub index: NodeIndex,
    pub children: Vec<RewardNodeConnection>,
    //pub parents: Vec<NodeIndex>,
    pub reward: Box<fn(&MapState, &CreatureState, &RequirementResult) -> RewardResult>,
    pub reward_connection: Box<fn(&MapState, &CreatureState, i32) -> f32>,
    pub requirement: Box<fn(&MapState, &CreatureState) -> RequirementResult>,
    pub cost: Box<fn(&MapState, &CreatureState, &RequirementResult) -> CostResult>,
    pub get_command: Option<Box<for<'f> fn(&'f MapState, &'f CreatureState, &RewardResult, &RequirementResult) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of an organizing node
    pub effect: Option<Box<fn(&MapState, &CreatureState, &RewardResult, &RequirementResult) -> Vec<VariableChange>>> // Used to get current of self already
}
pub struct RewardNodeCreatureList {
    pub description: String,  // just for debugging/comments
    pub index: NodeIndex,
    pub children: Vec<RewardNodeConnection>,
    //pub parents: Vec<usize>,
    pub reward: Box<fn(&MapState, &CreatureState, &RequirementResult, &CreatureState) -> RewardResult>,
    pub reward_connection: Box<fn(&MapState, &CreatureState, i32, &CreatureState) -> f32>,
    pub requirement: Box<fn(&MapState, &CreatureState, &CreatureState) -> RequirementResult>,
    pub cost: Box<fn(&MapState, &CreatureState, &RequirementResult, &CreatureState) -> CostResult>,
    pub get_command: Option<Box<for<'f> fn(&'f MapState, &'f CreatureState, &RewardResult, &RequirementResult, &'f CreatureState) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of an organizing node
    pub effect: Option<Box<fn(&MapState, &CreatureState, &RewardResult, &RequirementResult, &CreatureState) -> Vec<VariableChange>>>, // Used to get current of self already
    pub filter: Box<fn(&MapState, &CreatureState, &CreatureState)->bool>, // will take all known CreatureStates, then use this filter on them, to produce one NodeResult for each one.
}

pub struct RequirementResult{
    pub valid: bool,
    pub requirements: Vec<Vec<VariableChange>>, //requirements split by OR
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
pub struct RewardResult{
    pub reward_local: f32,
    // below can be used by other functions to do interesting stuff
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
pub struct CostResult{
    pub cost_base: f32,
    pub cost_divider: f32,
}
pub struct ConnectionResult {
    pub base_multiplier: f32, // multiplier based on just the requirements of child and effects of parent
    pub multiplier_child: f32, // child's Count based multiplier
    pub conn_reward: Option<f32>, // None when not set yet
    pub computed_count: i32, // used to compute the final reward multiplier for the child for his connection combined with bonus_count
    pub bonus_count: i32,
}

// Is typically just connectionResult, but with computed 
pub struct VariableReward{
    pub reward: f32, // for None category, this will be sum of all conn_results otherwise is the max for that category
    pub category: Variable,
}
pub struct NodeRewardGlobal{
    // local reward is stored in the RewardResult
    rewards_per_requirement: Vec<VariableReward>,
    reward_sum_total: Option<f32>, // includes local reward
    reward_global_with_costs: Option<f32>,
}

pub struct NodeResultRoot {
    pub nodes: Vec<NodeResult>,
    pub children: Vec<NodeIndex>,
    pub original_node_descriptor: String,
}
impl NodeResultRoot {
    pub fn calculate_global_reward(&mut self, root_node: &RootNode, map_state: &MapState, c_state: &CreatureState, c_target: Option<&CreatureState>, index_to_process: usize, indexes_processed: &mut HashSet<usize>) -> bool {
        if self.nodes[index_to_process].global_reward.reward_global_with_costs.is_some() {
            return true;
        }
        if indexes_processed.contains(&index_to_process) {
            eprintln!("There is a cycle in the graph (a nested child has a parent as their child), this means its impossible to compute! Last Node {} Nodes processed(random order): {:#?}", self.nodes[index_to_process].original_node_description.clone(), indexes_processed.iter().map(|i| {
                self.nodes[*i].original_node_description.clone()
            }));
            return false;
        }
        indexes_processed.insert(index_to_process);
        // go through all children and make sure they are calculated first.
        // depth first basically.
        for child in self.nodes[index_to_process].children.clone() {
            self.calculate_global_reward(root_node, map_state, c_state, c_target, child, indexes_processed);
        }

        // all children must have been processed now.
        // global reward sum is: reward_local + Sum(rewards_per_requirement)
        // final global reward with cost is: global_sum - cost_base / cost_multiplier
        
        let mut conn_by_categories: Vec<Vec<&RewardNodeConnection>> = vec![];
        let child_iter: &Vec<RewardNodeConnection> = match &root_node.nodes[index_to_process] {
            Node::Reward(r) => &r.children,
            Node::CreatureList(_) => todo!(),
        };
        
        child_iter.iter().for_each(|conn| {
            let mut existing = false;
            for cvec in conn_by_categories.iter_mut() {
                if cvec.len() > 0 && cvec[0].requirement.variable == conn.requirement.variable {
                    existing = true;
                    cvec.push(conn);
                    break;
                }
            }
            if !existing {
                conn_by_categories.push(vec![conn]);
            }
        });

        let mut conn_results: Vec<Vec<ConnectionResult>> = Vec::new();
        for conn_list in conn_by_categories.iter() {
            let mut cat_results: Vec<ConnectionResult> = vec![];
            let variable = conn_list[0].requirement.variable;

            if variable == Variable::None {
                // if its None, just get the reward and the multiplier calculation is with count 0 always?
                todo!();
            }

            let actual_count = get_count_of_variable(map_state, c_state, variable);
            for c in conn_list {
                let requirement_needed_in_child = get_variable_change_from_vec_vec(&self.nodes[c.child_index].requirement_result.requirements, variable);
                if requirement_needed_in_child == None {
                    eprintln!("have a Variable connection but child requirement does not actually have this Variable as its requirement. Graph is wrong! parent:{} child:{} variable:{:#?}", index_to_process, c.child_index, variable);
                    panic!("have a Variable connection but child requirement does not have requirement.");
                }
                let conn_requirement_needed = requirement_needed_in_child.unwrap().change as f32;
                let actual_parent_count = actual_count as f32 / conn_requirement_needed;
                // TODONEXT:
                // now just need to get "actually exists" of the child. this can be done by
                // for each of its effects, get their count, then for each variable get its 
                // reward proportion by finding its rewards_per_requirement/reward_sum_total
                let total_count = actual_parent_count + &self.nodes[c.child_index].get_count(map_state, c_state);
                assert!(total_count >= 0.);
                // if its none that means that node doesn't actually require this category?
                // or its None category? AGG maybe None requirement shouldn't exist!
                let base_multiplier = get_variable_change_from_effects(variable, &self.nodes[index_to_process].effects).unwrap().change as f32 / conn_requirement_needed;

                cat_results.push(ConnectionResult{
                    base_multiplier: base_multiplier,
                    multiplier_child: 1.,
                    conn_reward: Some(
                        get_global_reward_for_connection(
                            (&self).nodes.get((*c).child_index).unwrap(), 
                            root.nodes[(*c).child_index].global_reward.unwrap(),
                            base_multiplier,
                            total_count as i32, 
                            map_state, 
                            c_state,
                            None)
                        ),
                    computed_count: total_count as i32,
                    bonus_count: 0,
                });
                for i in 0..total_count as i32 {
                    // get the Max reward of them all, increase its count by 1 and recompute it.

                }
            }
            let count = get_count_of_variable(map_state, c_state, variable);
            

            conn_results.push(cat_results);
        }

        return true;
    }
}
pub struct NodeResult {
    pub original_node: NodeIndex,
    pub original_node_description: String,
    pub reward_result: RewardResult, //contains local reward
    pub cost_result: CostResult,
    pub requirement_result: RequirementResult,
    pub effects: Vec<VariableChange>,
    pub children: Vec<NodeIndex>,
    // Filled out as you do global reward:
    pub connection_results: Option<Vec<Vec<ConnectionResult>>>, // indexs should match children
    pub global_reward: NodeRewardGlobal,
    pub creature_target: Option<UID>,
}
impl NodeResult {
    pub fn get_count(&self, map_state: &MapState, c_state: &CreatureState) -> f32 {
        if self.global_reward.reward_sum_total.is_none() {
            panic!("Trying to get count when global reward has not been calculated yet!");
        }

        let total_sum = self.global_reward.reward_sum_total.unwrap();
        let mut total_count = 0.;
        for req_result in &self.global_reward.rewards_per_requirement {
            if req_result.category != Variable::None {
                total_count += (req_result.reward/total_sum) * get_count_of_variable(map_state, c_state, req_result.category) as f32;
            }
        }

        total_count
    }
}

