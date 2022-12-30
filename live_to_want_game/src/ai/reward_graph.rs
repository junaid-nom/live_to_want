use serde::{Deserialize, Serialize};
use std::collections::{HashSet, BinaryHeap};
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

pub fn get_global_reward_for_connection(child_multiplier: f32, global_reward: f32, base_multiplier: f32) -> f32 {
    return base_multiplier * child_multiplier * global_reward;
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

        root
    }
}

pub enum Node {
    Reward(RewardNode),
    CreatureList(RewardNodeCreatureList),
} impl Node {
    pub fn get_children(&self) -> &Vec<RewardNodeConnection> {
        match self {
            Node::Reward(r) => &r.children,
            Node::CreatureList(r) => &r.children,
        }
    }
    pub fn get_child_multiplier(&self, count: f32, m: &MapState, c: &CreatureState, c_target: Option<&CreatureState>) -> f32 {
        match self {
            Node::Reward(n) => n.reward_connection.as_ref()(m, c, count),
            Node::CreatureList(nl) => nl.reward_connection.as_ref()(m, c, count, c_target.unwrap()),
        }
    }
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
    pub base_multiplier: Option<f32>, // Needed for variable None connections. should be None and auto calculated via effects of parent node and requirements of child node
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
    pub reward_connection: Box<fn(&MapState, &CreatureState, f32) -> f32>,
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
    pub reward_connection: Box<fn(&MapState, &CreatureState, f32, &CreatureState) -> f32>,
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
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex,
    pub base_multiplier: f32, // multiplier based on just the requirements of child and effects of parent
    pub multiplier_child: f32, // child's Count based multiplier
    pub conn_reward: f32, // None when not set yet
    pub child_count: f32, // used to compute the final reward multiplier for the child for his connection combined with bonus_count
    pub parent_count: f32,
    pub parent_to_child_count_ratio: f32,
    pub category: Variable,
} 
impl PartialEq for ConnectionResult {
    fn eq(&self, other: &Self) -> bool {
        self.conn_reward == other.conn_reward
    }
}
impl Eq for ConnectionResult {}
impl PartialOrd for ConnectionResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.conn_reward.partial_cmp(&other.conn_reward);
    }
}
impl Ord for ConnectionResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.conn_reward.is_nan() {
            if other.conn_reward.is_nan() {
                return std::cmp::Ordering::Equal;
            } else {
                return std::cmp::Ordering::Less;
            }
        } else if other.conn_reward.is_nan() {
            return std::cmp::Ordering::Greater;
        }

        return self.conn_reward.partial_cmp(&other.conn_reward).unwrap();
    }
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

        let mut conn_results: Vec<BinaryHeap<ConnectionResult>> = Vec::new();
        for conn_list in conn_by_categories.iter() {
            let mut cat_results: BinaryHeap<ConnectionResult> = BinaryHeap::new();
            let variable = conn_list[0].requirement.variable;

            if variable == Variable::None {
                // For None category, we SUM all child connections.
                // Since they have no requirements connected to this node, we don't need
                // to do the LIMIT algorithm below or care about counts and other stuff
                let mut total_sum = 0.;
                // if its None, the count is 0, just get the base multiplier in the connection.
                for c in conn_list {
                    let base_mult = c.base_multiplier.unwrap();
                    let conn_result = ConnectionResult{
                        category: variable,
                        base_multiplier: base_mult,
                        multiplier_child: 1.,
                        conn_reward: get_global_reward_for_connection(1., self.nodes[(*c).child_index].global_reward.reward_global_with_costs.unwrap(), base_mult),
                        child_count: 0.,
                        parent_count: 0.,
                        parent_to_child_count_ratio: 0.,
                        child_index: c.child_index,
                        parent_index: c.parent_index,
                    };
                    total_sum+= conn_result.conn_reward;
                    cat_results.push(conn_result);
                }
                let final_max = VariableReward{
                    reward: total_sum,
                    category: variable,
                };
                self.nodes[index_to_process].global_reward.rewards_per_requirement.push(final_max);
                conn_results.push(cat_results);
                continue;
            }

            // If requirement for child connections are not None Variable then we need
            // to get the MAX connection reward, essentially if doing an action gives some wood and
            // some fiber, that actions reward would be (Best thing to do with wood) + (Best thing to do with fiber)

            // Getting the best thing to do is based on what items we already have
            // to figure this out below is the LIMIT ALGORITHM.
            // Imagine you need Wood as just 1 ingredient to create a bow.
            // If you have enough wood already for 10 bows, then the reward
            // for getting more wood should be reward for making an 11th bow.
            // Now to add to this, imagine we have two recipes requiring wood, bow and shield
            // To get the TRUE reward, we would have to recurssively simulate giving out the wood
            // one by one. So 1 wood to bow because its the highest reward, then the 2nd to 
            // shield. then the 3rd and 4th and 5th to shield because extra shields are more valuable
            // because shields break more often, but then 6th to Bow because we have so many shields.
            // It gets crazy when there nested rewards, so if Shield also is used to create SpikedShield
            // then we need to recussively check Shield's children based on different numbers of shield we have already.
            // thats computationally expensive and annoying to program so instead we Estimate 
            // the reward of additional items for an item with the reward_connection function
            // which takes in a Count as a parameter.
            let parent_count = get_count_of_variable(map_state, c_state, variable);
            let var_effect = get_variable_change_from_effects(variable, &self.nodes[index_to_process].effects).unwrap().change as f32;
            for c in conn_list {
                let requirement_needed_in_child = get_variable_change_from_vec_vec(&self.nodes[c.child_index].requirement_result.requirements, variable);
                if requirement_needed_in_child == None {
                    eprintln!("have a Variable connection but child requirement does not actually have this Variable as its requirement. Graph is wrong! parent:{} child:{} variable:{:#?}", index_to_process, c.child_index, variable);
                    panic!("have a Variable connection but child requirement does not have requirement.");
                }
                let conn_requirement_needed = requirement_needed_in_child.unwrap().change as f32;

                // now just need to get "actually exists" of the child. this can be done by
                // for each of its effects, get their count, then for each variable get its 
                // reward proportion by finding its rewards_per_requirement/reward_sum_total
                let child_count = self.nodes[c.child_index].get_count(map_state, c_state);
                assert!(child_count >= 0.);
                // if its none that means that node doesn't actually require this category?
                // or its None category? AGG maybe None requirement shouldn't exist!
                let base_multiplier = var_effect / conn_requirement_needed;
                assert!(c.base_multiplier.is_none()); // only None connections should have a hardCoded base multiplier

                let multiplier_child = (&root_node).nodes.get((*c).child_index).unwrap().get_child_multiplier(
                    child_count, 
                    map_state, 
                    c_state, 
                    None
                );
                cat_results.push(ConnectionResult{
                    category: variable,
                    base_multiplier: base_multiplier,
                    multiplier_child: multiplier_child,
                    conn_reward: get_global_reward_for_connection(
                        multiplier_child, 
                        self.nodes[(*c).child_index].global_reward.reward_global_with_costs.unwrap(),
                        base_multiplier,
                    ),
                    child_count: child_count,
                    parent_count: 0.,
                    parent_to_child_count_ratio: 1. / conn_requirement_needed,
                    child_index: c.child_index,
                    parent_index: c.parent_index,
                });
            }

            let increment_best_child = |mut top: ConnectionResult| -> ConnectionResult {
                top.parent_count += var_effect * top.parent_to_child_count_ratio;
                top.conn_reward = (&root_node).nodes.get(top.child_index).unwrap().get_child_multiplier(
                    top.child_count + top.parent_count, 
                    map_state,
                    c_state, 
                    None
                );
                top
            };
            for _ in 0..parent_count as i32 {
                // get the Max reward of them all, increase its count by 1 and recompute it.
                let best = increment_best_child(cat_results.pop().unwrap());
                cat_results.push(best);
            }
            // Now add one more that's essentially the reward for "What happens if I get one more of this node's effects"
            let best = increment_best_child(cat_results.pop().unwrap());
            let final_max = VariableReward{
                reward: best.conn_reward,
                category: best.category,
            };
            self.nodes[index_to_process].global_reward.rewards_per_requirement.push(final_max);

            conn_results.push(cat_results);
        }
        self.nodes[index_to_process].connection_results = Some(conn_results);

        let mut reward_sum_total = 0.;
        for var_result in &self.nodes[index_to_process].global_reward.rewards_per_requirement {
            reward_sum_total += var_result.reward;
        }
        self.nodes[index_to_process].global_reward.reward_sum_total = Some(reward_sum_total);
        let final_reward = (reward_sum_total - self.nodes[index_to_process].cost_result.cost_base) / self.nodes[index_to_process].cost_result.cost_divider;
        self.nodes[index_to_process].global_reward.reward_global_with_costs = Some(final_reward);
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
    pub connection_results: Option<Vec<BinaryHeap<ConnectionResult>>>,
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

