use serde::{Deserialize, Serialize};
use core::fmt;
use std::collections::{HashSet, BinaryHeap, HashMap};
use crate::{UID, MapState, CreatureState, CreatureCommand, Location, ItemType};

pub type NodeIndex = usize;

pub fn get_count_of_variable(m: &MapState, c: &CreatureState, v: Variable) -> i32 {
    // TODO get the count of each variable.
    // Most will be what does creature have in inventory.
    // others could be result of a function for example "whats my power level"
    // could even be something like "my rank in power compared to creatures near me"
    match v {
        Variable::None => 0,
        Variable::Bone => c.get_inventory_of_item(ItemType::Bone) as i32,
        Variable::Meat => c.get_inventory_of_item(ItemType::Meat) as i32,
        Variable::Berry => c.get_inventory_of_item(ItemType::Berry) as i32,
        Variable::Skin => c.get_inventory_of_item(ItemType::Skin) as i32,
        Variable::Wood => c.get_inventory_of_item(ItemType::Wood) as i32,
        Variable::Fiber => c.get_inventory_of_item(ItemType::Fiber) as i32,
        Variable::Spear => c.get_inventory_of_item(ItemType::Spear) as i32,
        Variable::Shield => c.get_inventory_of_item(ItemType::Shield) as i32,
        Variable::Arrow => c.get_inventory_of_item(ItemType::Arrow) as i32,
        Variable::Bow => c.get_inventory_of_item(ItemType::Bow) as i32,
    }
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

#[derive(Debug, Clone)]
pub struct RootNode {
    pub description: String,  // just for debugging/comments
    pub nodes: Vec<Node>,
    pub children: Vec<RewardNodeConnection>,
}
impl RootNode {
    pub fn generate_result_graph<'a>(&self, map_state :&'a MapState, c_state : &'a CreatureState) -> NodeResultRoot<'a> {
        let mut root = NodeResultRoot{
            children: vec![],
            original_node_descriptor: self.description.clone(),
            nodes: vec![],
        };

        let uid_map = map_state.get_creatures_hashmap();
        // TODO: Eventually the creatures remembered list should be culled based on frame
        // last updated, creatures memory (Trait to add) and distance. Also allow friendlies
        // to share each others remembered creatures.
        let creatures_remembered: Vec<&CreatureState> = c_state.memory.creatures_remembered.iter().map(|cr| {
            *uid_map.get(&cr.id).unwrap()
        }).collect();
        // TODO also add creatures currently visible to the creatures to this list

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
                    let cmd: Option<CreatureCommand> = match &n.get_command {
                        Some(fun) => Some(fun.as_ref()(map_state, c_state,&reward, &requirement)),
                        None => None,
                    };
                    let new = NodeResult {
                        original_node_description: n.description.clone(), // NOTE: Is this worth? Every frame copying over the description. All to make debug print easier?
                        requirement_result: requirement,
                        reward_result: reward,
                        cost_result: cost,
                        global_reward: NodeRewardGlobal { rewards_per_result_change: vec![], reward_sum_total: None, reward_global_with_costs: None },
                        children: (&n.children).into_iter().map(|c| c.child_index).collect(),
                        effects: effects,
                        connection_results: None, // need to wait for global results of children to compute this
                        original_node: n.index,
                        creature_target: None,
                        command_result: cmd,
                    };
                    root.nodes.push(new);
                },
                Node::CreatureList(nl) => {
                    // get the filtered list of targets. then make a NodeResult for all of them
                    let filter = |target: &CreatureState| -> bool {
                        nl.filter.as_ref()(map_state, c_state, target)
                    };
                    let mut targets = vec![];
                    for c in &creatures_remembered {
                        if filter(c) {
                            targets.push(*c);
                        }
                    }
                    for target in targets {
                        // make a NodeResult for em
                        let requirement = nl.requirement.as_ref()(map_state, c_state, target);
                        let reward = nl.reward.as_ref()(map_state, c_state, &requirement, target);
                        let cost = nl.cost.as_ref()(map_state, c_state, &requirement, target);
                        let effects = match &nl.effect {
                            Some(e) => e.as_ref()(map_state, c_state, &reward, &requirement, target),
                            None => vec![],
                        };
                        let cmd: Option<CreatureCommand> = match &nl.get_command {
                            Some(fun) => Some(fun.as_ref()(map_state, c_state,&reward, &requirement, target)),
                            None => None,
                        };
                        let new = NodeResult {
                            original_node_description: format!("{}:{}", nl.description.clone(), target.get_id()), // NOTE: Is this worth? Every frame copying over the description. All to make debug print easier?
                            requirement_result: requirement,
                            reward_result: reward,
                            cost_result: cost,
                            global_reward: NodeRewardGlobal { rewards_per_result_change: vec![], reward_sum_total: None, reward_global_with_costs: None },
                            children: (&nl.children).into_iter().map(|c| c.child_index).collect(),
                            effects: effects,
                            connection_results: None, // need to wait for global results of children to compute this
                            original_node: nl.index,
                            creature_target: Some(target.get_id()),
                            command_result: cmd,
                        };
                        root.nodes.push(new);
                    }
                },
            }
        }

        // setup children connections for root
        root.children = vec![];
        for result_index in 0..root.nodes.len() {
            for root_child in &self.children {
                println!("resultIndex: {} childIndex:{}", root.nodes[result_index].original_node, root_child.child_index);
                if root.nodes[result_index].original_node == root_child.child_index {
                    root.children.push(result_index);
                }
            }
        }

        // Now how to get the reward of everything...
        // I guess: go through from root, if node doesn't have global reward set yet, then, calculate the global reward on it.
        // First step of that is to calculate it on its children recurssively
        for i in 0..root.children.len() {
            let mut indexs_processed = HashSet::new();
            // Converting the UID of creature target, to the creature state itself
            // is gonna be REALLY confusing. Couldn't just be a reference to 
            // a COPY of the CreatureState in the MEMORY of the Creature doing this AI,
            // because then will need to update the memory for all creatures currently in view constantly.
            // ok so can't do that copy idea.
            // instead make a hashmap of UID->CreatureState from map_state
            // creature memory just stores UID and last location seen? only of important friendly stuff?
            root.calculate_global_reward( &self, map_state, c_state, &uid_map, root.children[i], &mut indexs_processed);
        }

        root
    }
}
#[derive(Debug, Clone)]
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
    Meat,
    Skin,
    Berry,
    Wood,
    Fiber,
    Spear,
    Shield,
    Arrow,
    Bow,
    // NOTE inbetween ingredients will need to be variables. Anything that is an inner OR. For example, if  (wood OR clay) AND glue makes a wall, then (wood OR clay) must be its own node and variable.
}
#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct VariableChange {
    pub variable: Variable,
    pub change: i32,
}
#[derive(Clone, Debug)]
pub struct RewardNodeConnection {
    pub base_multiplier: Option<f32>, // Needed for variable None connections. should be None and auto calculated via effects of parent node and requirements of child node
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex, // debug only
    pub requirement: VariableChange, // multiplier is: 1/requirement.change * effect(for that variable)
}
#[derive(Clone)]
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
} impl fmt::Debug for RewardNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewardNode")
         .field("description", &self.description)
         .field("index", &self.index)
         .field("children", &self.children)
         .field("get_command", &self.get_command.is_some())
         .field("effect", &self.effect.is_some())
         .finish()
    }
}
#[derive(Clone)]
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
} impl fmt::Debug for RewardNodeCreatureList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewardNodeCreatureList")
         .field("description", &self.description)
         .field("index", &self.index)
         .field("children", &self.children)
         .field("get_command", &self.get_command.is_some())
         .field("effect", &self.effect.is_some())
         .finish()
    }
}

#[derive(Clone, Debug)]
pub struct RequirementResult{
    pub valid: bool,
    pub requirements: Vec<Vec<VariableChange>>, //requirements split by OR
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
#[derive(Clone, Debug)]
pub struct RewardResult{
    pub reward_local: f32,
    // below can be used by other functions to do interesting stuff
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
#[derive(Clone, Debug)]
pub struct CostResult{
    pub cost_base: f32,
    pub cost_divider: f32,
}

#[derive(Clone, Debug)]
pub struct ConnectionResult {
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex,
    pub base_multiplier: f32, // multiplier based on just the requirements of child and effects of parent
    pub multiplier_child: f32, // child's Count based multiplier
    pub total_reward: Vec<f32>, // empty when not set. will push to the top of list for every reward calculated. total_reward[0] is the final calculated one
    pub child_count: f32, // used to compute the final reward multiplier for the child for his connection combined with bonus_count
    pub parent_count: f32,
    pub parent_count_total: i32,
    pub parent_to_child_count_ratio: f32,
    pub category: Variable,
} 
impl PartialEq for ConnectionResult {
    fn eq(&self, other: &Self) -> bool {
        self.total_reward == other.total_reward
    }
}
impl Eq for ConnectionResult {}
impl PartialOrd for ConnectionResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.total_reward.partial_cmp(&other.total_reward);
    }
}
impl Ord for ConnectionResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.total_reward.len() == 0 || self.total_reward[0].is_nan() {
            if other.total_reward.len() == 0 || other.total_reward[0].is_nan() {
                return std::cmp::Ordering::Equal;
            } else {
                return std::cmp::Ordering::Less;
            }
        } else if other.total_reward.len() == 0 || other.total_reward[0].is_nan() {
            return std::cmp::Ordering::Greater;
        }

        if self.total_reward[0] == other.total_reward[0] { 
            return (-1 * self.child_index as i32).cmp(&(-1 * other.child_index as i32));
        }
        return self.total_reward[0].partial_cmp(&other.total_reward[0]).unwrap();
    }
}

#[derive(Clone, Debug, PartialEq)]
// Is typically just connectionResult, but with computed 
pub struct VariableReward{
    pub reward: f32, // for None category, this will be sum of all conn_results otherwise is the max for that category
    pub category: Variable,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NodeRewardGlobal{
    // local reward is stored in the RewardResult
    pub rewards_per_result_change: Vec<VariableReward>,
    pub reward_sum_total: Option<f32>, // includes local reward
    pub reward_global_with_costs: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct NodeResultRoot<'f> {
    pub nodes: Vec<NodeResult<'f>>,
    pub children: Vec<NodeIndex>,
    pub original_node_descriptor: String,
}
impl NodeResultRoot<'_> {
    pub fn calculate_global_reward(&mut self, og_root_node: &RootNode, map_state: &MapState, c_state: &CreatureState, c_targets: &HashMap<UID, &CreatureState>, index_to_process: usize, indexes_processed: &mut HashSet<usize>) -> bool {
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
            self.calculate_global_reward(og_root_node, map_state, c_state, c_targets, child, indexes_processed);
        }

        let c_target = match self.nodes[index_to_process].creature_target  {
            Some(target) => Some(*c_targets.get(&target).unwrap()),
            None => None,
        };
        // all children must have been processed now.
        // global reward sum is: reward_local + Sum(rewards_per_requirement)
        // final global reward with cost is: global_sum - cost_base / cost_multiplier
        
        let mut conn_by_categories: Vec<Vec<&RewardNodeConnection>> = vec![];
        let original_node_index = self.nodes[index_to_process].original_node;
        let child_iter: &Vec<RewardNodeConnection> = og_root_node.nodes[original_node_index].get_children();
        
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
                        total_reward: vec![get_global_reward_for_connection(1., self.nodes[(*c).child_index].global_reward.reward_global_with_costs.unwrap(), base_mult)],
                        child_count: 0.,
                        parent_count: 0.,
                        parent_count_total: 0,
                        parent_to_child_count_ratio: 0.,
                        child_index: c.child_index,
                        parent_index: c.parent_index,
                    };
                    total_sum+= conn_result.total_reward[0];
                    cat_results.push(conn_result);
                }
                let final_max = VariableReward{
                    reward: total_sum,
                    category: variable,
                };
                self.nodes[index_to_process].global_reward.rewards_per_result_change.push(final_max);
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

                let multiplier_child = (&og_root_node).nodes.get((*c).child_index).unwrap().get_child_multiplier(
                    child_count, 
                    map_state, 
                    c_state, 
                    c_target
                );
                cat_results.push(ConnectionResult{
                    category: variable,
                    base_multiplier: base_multiplier,
                    multiplier_child: multiplier_child,
                    total_reward: vec![get_global_reward_for_connection(
                        multiplier_child, 
                        self.nodes[(*c).child_index].global_reward.reward_global_with_costs.unwrap(),
                        base_multiplier,
                    )],
                    child_count: child_count,
                    parent_count: 0.,
                    parent_count_total: parent_count,
                    parent_to_child_count_ratio: 1. / conn_requirement_needed,
                    child_index: c.child_index,
                    parent_index: c.parent_index,
                });
            }

            let increment_best_child = |mut top: ConnectionResult| -> ConnectionResult {
                top.parent_count += var_effect * top.parent_to_child_count_ratio;
                top.multiplier_child = (&og_root_node).nodes.get(top.child_index).unwrap().get_child_multiplier(
                    top.child_count + top.parent_count, 
                    map_state,
                    c_state, 
                    c_target
                );
                top.total_reward.insert(0, get_global_reward_for_connection(
                    top.multiplier_child, 
                    self.nodes[top.child_index].global_reward.reward_global_with_costs.unwrap(),
                    top.base_multiplier,
                ));
                top
            };
            for _ in 0..parent_count as i32 {
                // get the Max reward of them all, increase its count by 1 and recompute it.
                let best = increment_best_child(cat_results.pop().unwrap());
                cat_results.push(best);
            }
            // Now add one more that's essentially the reward for "What happens if I get one more of this node's effects"
            let best = cat_results.pop().unwrap();
            let final_max = VariableReward{
                reward: best.total_reward[0],
                category: best.category,
            };
            cat_results.push(best);
            self.nodes[index_to_process].global_reward.rewards_per_result_change.push(final_max);

            conn_results.push(cat_results);
        }
        self.nodes[index_to_process].connection_results = Some(conn_results);

        let mut reward_sum_total = self.nodes[index_to_process].reward_result.reward_local;
        for var_result in &self.nodes[index_to_process].global_reward.rewards_per_result_change {
            reward_sum_total += var_result.reward;
        }
        self.nodes[index_to_process].global_reward.reward_sum_total = Some(reward_sum_total);
        let final_reward = (reward_sum_total - self.nodes[index_to_process].cost_result.cost_base) / self.nodes[index_to_process].cost_result.cost_divider;
        self.nodes[index_to_process].global_reward.reward_global_with_costs = Some(final_reward);
        return true;
    }

    pub fn get_final_command<'b, 'l>(&'l self) -> Option<CreatureCommand<'l>> { 
        let mut best_node: Option<&NodeResult> = None;
        for item in self.nodes.iter() {
            if 
                item.requirement_result.valid && 
                item.command_result.is_some() && 
                (
                    best_node.is_none()
                    ||
                    item.global_reward.reward_global_with_costs.unwrap() >= 
                    best_node.unwrap().global_reward.reward_global_with_costs.unwrap()
                )
                 {
                    best_node = Some(item);
            }
        }
        
        if let Some(best_node) = best_node {
            return Some(best_node.command_result.clone().unwrap());
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct NodeResult<'f> {
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
    pub command_result: Option<CreatureCommand<'f>>,
    pub creature_target: Option<UID>,
}
impl NodeResult<'_> {
    pub fn get_count(&self, map_state: &MapState, c_state: &CreatureState) -> f32 {
        if self.global_reward.reward_sum_total.is_none() {
            panic!("Trying to get count when global reward has not been calculated yet!");
        }

        // get reward from children and calculate count based on contribution to total reward
        let total_sum = self.global_reward.reward_sum_total.unwrap();
        let mut total_count = 0.;
        for var_result in &self.global_reward.rewards_per_result_change {
            if var_result.category != Variable::None {
                total_count += (var_result.reward/total_sum) * get_count_of_variable(map_state, c_state, var_result.category) as f32;
            }
        }

        // now get local reward contribution and add that, assuming all effects have equal weight per count
        // if effect of this is: +10 X and +3 Y means every X has 1/13 of total local contribution for simplicity.
        let local_proportion = self.reward_result.reward_local / total_sum;
        if local_proportion > 0. {
            let mut total_effect = 0;
            for effect in &self.effects {
                total_effect += effect.change;
            }
            for effect in &self.effects {
                let count =  get_count_of_variable(map_state, c_state, effect.variable);
                total_count += count as f32 / total_effect as f32 * local_proportion;
            }
        }

        total_count
    }
}

