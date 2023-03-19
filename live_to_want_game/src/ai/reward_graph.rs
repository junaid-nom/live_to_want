use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use core::fmt;
use std::{collections::{HashSet, BinaryHeap, HashMap}, hash::Hash, fmt::format};
use crate::{UID, MapState, CreatureState, CreatureCommand, Location, ItemType};

pub type NodeIndex = usize;
pub type NodeResultIndex = usize;

pub fn get_count_of_effect(_m: &MapState, c: &CreatureState, e: Effect) -> i32 {
    // get the count of each effect.
    // Most will be what does creature have in inventory.
    // others could be result of a function for example "whats my power level"
    // could even be something like "my rank in power compared to creatures near me"
    match e {
        Effect::None => 0,
        Effect::DropItem(item) => c.get_inventory_of_item(item) as i32,
        Effect::HaveItem(item) => c.get_inventory_of_item(item) as i32,
    }
}

pub fn get_global_reward_for_connection(child_multiplier: f32, global_reward: f32, base_multiplier: f32) -> f32 {
    return base_multiplier * child_multiplier * global_reward;
}

pub fn get_smallest_effect_change_from_vec_vec(changes :&Vec<Vec<EffectChange>>, effect: Effect) -> Option<EffectChange> {
    let mut lowest_effect_change: Option<EffectChange> = None;

    for ec_list in changes {
        for ec in ec_list {
            if ec.effect == effect {
                if lowest_effect_change.is_none() || ec.change < lowest_effect_change.unwrap().change {
                    lowest_effect_change = Some(*ec);
                } 
            }
        }
    }

    lowest_effect_change
}

pub fn get_effect_change_from_effects(category: Effect, effects: &Vec<EffectChange>) -> Option<EffectChange> {
    for ec in effects.iter() {
        if ec.effect == category {
            return Some(*ec);
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
    pub fn generate_result_graph<'a>(&self, map_state :&'a MapState, c_state : &'a CreatureState, uid_map: &HashMap<UID, &CreatureState>) -> NodeResultRoot {
        let mut root = NodeResultRoot{
            children: vec![],
            original_node_descriptor: self.description.clone(),
            nodes: vec![],
            requirement_map: HashMap::new(),
            
            creature_id: c_state.get_id(),
            frame: map_state.frame_count,
            final_node_descriptor: None,
        };

        // TODO: Eventually the creatures remembered list should be culled based on frame
        // last updated, creatures memory (Trait to add) and distance. Also allow friendlies
        // to share each others remembered creatures.
        let mut creatures_remembered: Vec<&CreatureState> = c_state.memory.creatures_remembered.iter().map(|cr| {
            *uid_map.get(&cr.id).unwrap()
        }).collect();
        if let Some(vc) = &c_state.components.vision_component {
            for seen in vc.visible_creatures.iter() {
                creatures_remembered.push(uid_map.get(seen).unwrap());
            }
        }

        // Add notable locations here to targets. For now just locations with items in them.
        let mut notable_locations = HashSet::new();
        let c_loc = c_state.get_location();
        for (_, location) in map_state.get_ground_item_list_for_region(c_state.get_location().region) {
            if !notable_locations.contains(&location) && c_loc.distance_in_region(&location).unwrap() as f32 <= c_state.get_vision_range() {
                notable_locations.insert(location);
            }
        }

        let mut result_to_og_idx_map: HashMap<NodeIndex, Vec<NodeResultIndex>> = HashMap::new();

        for node in &self.nodes {
            match node{
                Node::Reward(n) => {
                    let mut requirement = n.requirement.as_ref()(map_state, c_state);
                    requirement.dynamic_and_static_requirements.extend(n.static_requirements.clone());
                    let reward = n.reward.as_ref()(map_state, c_state, &requirement);
                    let cost = n.cost.as_ref()(map_state, c_state, &requirement);
                    let effects = match &n.effect {
                        Some(e) => e.as_ref()(map_state, c_state, &reward, &requirement),
                        None => vec![],
                    };
                    let idx = root.nodes.len();
                    let new = NodeResult {
                        index: idx,
                        original_node_description: n.description.clone(), // NOTE: Is this worth? Every frame copying over the description. All to make debug print easier?
                        requirement_result: requirement,
                        reward_result: reward,
                        cost_result: cost,
                        global_reward: NodeRewardGlobal { rewards_per_result_change: vec![], reward_sum_total: None, reward_global_with_costs: None },
                        children_result: vec![], // (&n.static_children).into_iter().map(|c| c.child_index).collect()
                        effects: effects,
                        connection_results: None, // need to wait for global results of children to compute this
                        original_node_index: n.index,
                        list_target: None,
                        has_command: n.get_command.is_some(),
                    };
                    root.nodes.push(new);
                    result_to_og_idx_map.insert(n.index, vec![idx]);
                },
                Node::ListNode(nl) => {
                    result_to_og_idx_map.insert(nl.index, vec![]);
                    // get the filtered list of targets. then make a NodeResult for all of them
                    let filter = |target: &NodeTarget| -> bool {
                        nl.filter.as_ref()(map_state, c_state, target)
                    };
                    let mut targets: Vec<NodeTarget> = vec![];
                    for target_type in &nl.target_types {
                        match target_type {
                            NodeTargetType::CreatureTarget => {
                                for c in &creatures_remembered {
                                    let target = NodeTarget::CreatureTarget(c);
                                    if filter(&target) {
                                        targets.push(target);
                                    }
                                }
                            },
                            NodeTargetType::LocationItemTarget => {
                                for loc in &notable_locations {
                                    for item in &map_state[loc].items {
                                        let target = NodeTarget::LocationItemTarget(*loc, item.item_type);
                                        if filter(&target) {
                                            targets.push(target);
                                        }
                                    }
                                    
                                }
                            },
                        }
                    }
                    
                    for target in &targets {
                        // make a NodeResult for em
                        let mut requirement = nl.requirement.as_ref()(map_state, c_state, target);
                        requirement.dynamic_and_static_requirements.extend(nl.static_requirements.clone());
                        let reward = nl.reward.as_ref()(map_state, c_state, &requirement, target);
                        let cost = nl.cost.as_ref()(map_state, c_state, &requirement, target);
                        let effects = match &nl.effect {
                            Some(e) => e.as_ref()(map_state, c_state, &reward, &requirement, target),
                            None => vec![],
                        };
                        let idx = root.nodes.len();
                        let new = NodeResult {
                            index: idx,
                            original_node_description: format!("{}:{:#?}", nl.description.clone(), target.get_id()), // NOTE: Is this worth? Every frame copying over the description. All to make debug print easier?
                            requirement_result: requirement,
                            reward_result: reward,
                            cost_result: cost,
                            global_reward: NodeRewardGlobal { rewards_per_result_change: vec![], reward_sum_total: None, reward_global_with_costs: None },
                            children_result: vec![], // (&nl.static_children).into_iter().map(|c| c.child_index).collect()
                            effects: effects,
                            connection_results: None, // need to wait for global results of children to compute this
                            original_node_index: nl.index,
                            list_target: Some(target.get_id()),
                            has_command: nl.get_command.is_some(),
                        };
                        root.nodes.push(new);
                        result_to_og_idx_map.get_mut(&nl.index).unwrap().push(idx);
                    }
                },
            }
        }

        // Setup children for all nodes now that we have all nodes effects and static requirements.
        // need to get the req map first
        for node_result in &root.nodes {
            let reqs = &node_result.requirement_result.dynamic_and_static_requirements;
            let mut added_effects = HashSet::new();
            for option_or in reqs.iter() {
                for change in option_or {
                    // Don't include None requirements. I guess it would be bizarre to have that tho anyway.
                    if added_effects.contains(&change.effect) || change.effect == Effect::None {
                        continue;
                    }
                    added_effects.insert(change.effect);

                    if root.requirement_map.contains_key(&change.effect) {
                        if !root.requirement_map.get(&change.effect).unwrap().contains(&node_result.original_node_index) {
                            root.requirement_map.get_mut(&change.effect).unwrap().push(node_result.original_node_index);
                        }
                    } else {
                        root.requirement_map.insert(change.effect, vec![node_result.original_node_index]);
                    }
                }
            }
        }

        // Need to do some annoying shenanigans here, do stuff with root immutable ref.
        // then mutate the stuff in root.
        let mut children_conns = vec![];
        // make sure all refs to children are expecting them to be the RESULT node's index not original.
        for node_result in &root.nodes {
            let result_conns = self.nodes.get(node_result.original_node_index).unwrap()
                .get_children(self, &root.requirement_map, &node_result.effects).into_iter().map(|conn_og| {
                    let mut results = vec![];
                    for result_child_idx in result_to_og_idx_map.get(&conn_og.child_index).unwrap() {
                        // if its a creatureList->creatureList, only save connections that share target
                        // creature!
                        let child_result_node = root.nodes.get(*result_child_idx).unwrap();
                        // for creatureListNode being child to another creatureListNode usually only want
                        // to do the connection if they have a matching target.
                        if node_result.list_target.is_none() || child_result_node.list_target.is_none() || node_result.list_target.as_ref().unwrap() == child_result_node.list_target.as_ref().unwrap() || conn_og.dont_match_targets {
                            // Transform og index to result indexes
                            let mut result_conn = conn_og.clone();
                            result_conn.parent_index = node_result.index;
                            result_conn.child_index = *result_child_idx;
                            results.push(result_conn);
                        }
                    }
                    results
                }).flatten().collect();
            children_conns.push(result_conns);
        }
        assert_eq!(children_conns.len(), root.nodes.len());
        children_conns.into_iter().enumerate().for_each(|(i, conn_list)| {
            let node_result = &mut root.nodes[i];
            node_result.children_result = conn_list;
        });
        

        // setup children connections for root
        root.children = vec![];
        for result_index in 0..root.nodes.len() {
            for root_child in &self.children {
                //println!("original index of result node: {} childIndex:{}", root.nodes[result_index].original_node_index, root_child.child_index);
                if root.nodes.get(result_index).unwrap().original_node_index == root_child.child_index {
                    root.children.push(result_index);
                }
            }
        }

        // Now how to get the reward of everything...
        // I guess: go through from root, if node doesn't have global reward set yet, then, calculate the global reward on it.
        // First step of that is to calculate it on its children recurssively
        for i in 0..root.children.len() {
            let mut indexs_processed = HashSet::new();
            // make a hashmap of UID->CreatureState from map_state
            root.calculate_global_reward( &self, map_state, c_state, &uid_map, root.children[i], &mut indexs_processed);
        }

        root
    }
}
#[derive(Debug, Clone)]
pub enum Node {
    Reward(RewardNode),
    ListNode(RewardNodeList),
} impl Node {
    pub fn get_children(&self, _root: &RootNode, req_map: &HashMap<Effect, Vec<NodeIndex>>, effects: &Vec<EffectChange>) -> Vec<RewardNodeConnection> {
        let mut total_conns: Vec<RewardNodeConnection> = vec![];
        let mut added_already: HashSet<NodeIndex> = HashSet::new();
        // add static connections
        match self {
            Node::Reward(r) => r.static_children.iter().for_each(|conn| {
                assert!(!added_already.contains(&conn.child_index));
                added_already.insert(conn.child_index);
                total_conns.push(conn.clone());
            }),
            Node::ListNode(r) => r.static_children.iter().for_each(|conn| {
                assert!(!added_already.contains(&conn.child_index));
                added_already.insert(conn.child_index);
                total_conns.push(conn.clone());
            }),
        };
        //println!("Getting child req_map:{:#?}", req_map);
        //println!("Getting child effects:{:#?}", effects);
        // add dynamic connections if they aren't already added
        effects.iter().for_each(|effect| {
            if let Some(dyn_conns) = req_map.get(&effect.effect) {
                for child in dyn_conns {
                    if !added_already.contains(child) {
                        added_already.insert(*child);
                        total_conns.push(RewardNodeConnection {
                            base_multiplier: None,
                            child_index: *child,
                            parent_index: self.get_index(),
                            category: effect.effect,
                            dont_match_targets: true, // not sure about this. but shouldn't matter usually. for the one test I got that is listnode->listnode (test_creature_list_node_reward_graph_2layer) it needs this to be true.
                        });
                    }
                }
            }
        });

        total_conns
    }
    pub fn get_child_multiplier(&self, count: f32, m: &MapState, c: &CreatureState, target: Option<&NodeTarget>) -> f32 {
        match self {
            Node::Reward(n) => n.reward_connection.as_ref()(m, c, count),
            Node::ListNode(nl) => nl.reward_connection.as_ref()(m, c, count, target.unwrap()),
        }
    }

    pub fn get_index(&self) -> NodeIndex {
        match self {
            Node::Reward(n) => n.index,
            Node::ListNode(nl) => nl.index,
        }
    }

    pub fn get_static_requirements(&self) -> &Vec<Vec<EffectChange>> {
        match self {
            Node::Reward(n) => &n.static_requirements,
            Node::ListNode(nl) => &nl.static_requirements,
        }
    }

}

#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub enum Effect {
    None,
    DropItem(ItemType), // produces an item on the ground (kill a creature etc)
    HaveItem(ItemType), // pickup or craft
    // NOTE inbetween ingredients will need to be effects. Anything that is an inner OR. For example, if  (wood OR clay) AND glue makes a wall, then (wood OR clay) must be its own node and effect.
}
#[derive(Debug)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
pub struct EffectChange {
    pub effect: Effect,
    pub change: i32,
}
impl EffectChange {
    pub fn new(effect: Effect, change: i32) -> Self {
        EffectChange { effect, change }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RewardNodeConnection {
    pub base_multiplier: Option<f32>, // Needed for effect None connections. should be None and auto calculated via effects of parent node and requirements of child node. unused for connections based on effects even static ones
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex, // debug only
    pub category: Effect,
    pub dont_match_targets: bool, // Normally when ListNode is child to another ListNode then it should only match the connections based on target matching. For example: Move node -> kill node. reward of the move node for creature 1 is based on the kill node ONLY for creature 1 as well not all creatures. setting this to true will disable that (so far only used for tests?). 
}
#[derive(Clone)]
pub struct RewardNode {
    pub description: String,  // just for debugging/comments
    pub index: NodeIndex,
    pub static_children: Vec<RewardNodeConnection>,
    pub static_requirements: Vec<Vec<EffectChange>>,
    //pub parents: Vec<NodeIndex>,
    pub reward: Box<fn(&MapState, &CreatureState, &RequirementResult) -> RewardResult>,
    pub reward_connection: Box<fn(&MapState, &CreatureState, f32) -> f32>,
    pub requirement: Box<fn(&MapState, &CreatureState) -> RequirementResult>,
    pub cost: Box<fn(&MapState, &CreatureState, &RequirementResult) -> CostResult>,
    pub get_command: Option<Box<for<'f> fn(&'f MapState, &'f CreatureState, &RewardResult, &RequirementResult) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of an organizing node
    pub effect: Option<Box<fn(&MapState, &CreatureState, &RewardResult, &RequirementResult) -> Vec<EffectChange>>> // Used to get current of self already
} impl fmt::Debug for RewardNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewardNode")
         .field("description", &self.description)
         .field("index", &self.index)
         .field("children", &self.static_children)
         .field("get_command", &self.get_command.is_some())
         .field("effect", &self.effect.is_some())
         .finish()
    }
}

#[derive(Clone)]
// must be 1:1 with NodeTargetID
pub enum NodeTarget<'a> {
    CreatureTarget(&'a CreatureState),
    LocationItemTarget(Location, ItemType),
}
impl<'a> NodeTarget<'a> {
    pub fn get_id(&self) -> NodeTargetID {
        match self {
            NodeTarget::CreatureTarget(c) => NodeTargetID::CreatureTarget(c.get_id()),
            NodeTarget::LocationItemTarget(loc, item) => NodeTargetID::LocationItemTarget(*loc, *item),
        }
    }

    pub fn as_creature(& self) -> &'a CreatureState {
        match self {
            NodeTarget::CreatureTarget(c) => c,
            _ => panic!("Unwraping as creaturestate when target is not creature target"),
        }
    }

    pub fn as_location_item(& self) -> (&Location, &ItemType) {
        match self {
            NodeTarget::LocationItemTarget(loc, itype) => (loc, itype),
            _ => panic!("Unwraping as creaturestate when target is not creature target"),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
// must be 1:1 with NodeTarget
pub enum NodeTargetID { 
    CreatureTarget(UID),
    LocationItemTarget(Location, ItemType),
} impl NodeTargetID {
    pub fn get_node_target<'l>(&self, c_targets: &'l HashMap<UID, &CreatureState>) -> NodeTarget<'l> {
        match &self {
            NodeTargetID::CreatureTarget(c_uid) => NodeTarget::CreatureTarget(*c_targets.get(&c_uid).unwrap()),
            NodeTargetID::LocationItemTarget(loc, item) => NodeTarget::LocationItemTarget(*loc, *item),
        }
    }
 }
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum NodeTargetType { 
    CreatureTarget,
    LocationItemTarget,
}

#[derive(Clone)]
pub struct RewardNodeList {
    pub description: String,  // just for debugging/comments
    pub index: NodeIndex,
    pub static_children: Vec<RewardNodeConnection>,
    pub static_requirements: Vec<Vec<EffectChange>>,
    //pub parents: Vec<usize>,
    pub reward: Box<fn(&MapState, &CreatureState, &RequirementResult, &NodeTarget) -> RewardResult>,
    pub reward_connection: Box<fn(&MapState, &CreatureState, f32, &NodeTarget) -> f32>,
    pub requirement: Box<fn(&MapState, &CreatureState, &NodeTarget) -> RequirementResult>,
    pub cost: Box<fn(&MapState, &CreatureState, &RequirementResult, &NodeTarget) -> CostResult>,
    pub get_command: Option<Box<for<'f> fn(&'f MapState, &'f CreatureState, &RewardResult, &RequirementResult, NodeTarget<'f>) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of an organizing node
    pub effect: Option<Box<fn(&MapState, &CreatureState, &RewardResult, &RequirementResult, &NodeTarget) -> Vec<EffectChange>>>, // Used to get current of self already
    pub filter: Box<fn(&MapState, &CreatureState, &NodeTarget)->bool>, // will take all known targets of the valid types, then use this filter on them, to produce one NodeResult for each one.
    pub target_types: HashSet<NodeTargetType>, // invalid NodeTargetID that are just meant to denote what NodeTarget types to use for this node
} impl fmt::Debug for RewardNodeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RewardNodeCreatureList")
         .field("description", &self.description)
         .field("index", &self.index)
         .field("children", &self.static_children)
         .field("static_requirements", &self.static_requirements)
         .field("get_command", &self.get_command.is_some())
         .field("effect", &self.effect.is_some())
         .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequirementResult{
    pub valid: bool,
    pub dynamic_and_static_requirements: Vec<Vec<EffectChange>>, //requirements split by OR // total requirements are actually the dynamic requirements extended by the static requirements. dynamic ones are generated by the requirements function and are rare, most should be static.
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RewardResult{
    pub reward_local: f32,
    // below can be used by other functions to do interesting stuff
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CostResult{
    pub cost_base: f32,
    pub cost_divider: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionResult {
    pub child_index_node_result: NodeResultIndex,
    pub parent_index_node_result: NodeResultIndex,
    pub base_multiplier: f32, // multiplier based on just the requirements of child and effects of parent
    pub multiplier_child: f32, // child's Count based multiplier
    pub total_reward: Vec<f32>, // empty when not set. will push to the top of list for every reward calculated. total_reward[0] is the final calculated one
    pub child_count: f32, // used to compute the final reward multiplier for the child for his connection combined with bonus_count
    pub parent_count: f32,
    pub parent_count_total: i32,
    pub parent_to_child_count_ratio: f32,
    pub category: Effect,
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
            return (-1 * self.child_index_node_result as i32).cmp(&(-1 * other.child_index_node_result as i32));
        }
        return self.total_reward[0].partial_cmp(&other.total_reward[0]).unwrap();
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
// Is typically just connectionResult, but with computed 
pub struct EffectReward{
    pub reward: f32, // for None category, this will be sum of all conn_results otherwise is the max for that category
    pub category: Effect,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NodeRewardGlobal{
    // local reward is stored in the RewardResult
    pub rewards_per_result_change: Vec<EffectReward>,
    pub reward_sum_total: Option<f32>, // includes local reward
    pub reward_global_with_costs: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeResultRoot {
    pub nodes: Vec<NodeResult>,
    pub children: Vec<NodeIndex>,
    pub original_node_descriptor: String,
    pub requirement_map: HashMap<Effect, Vec<NodeIndex>>,

    // Debug vars:
    pub creature_id: UID,
    pub frame: u128,
    pub final_node_descriptor: Option<String>,
}
impl NodeResultRoot {
    pub fn calculate_global_reward(&mut self, og_root_node: &RootNode, map_state: &MapState, c_state: &CreatureState, c_targets: &HashMap<UID, &CreatureState>, index_to_process: usize, indexes_processed: &mut HashSet<usize>) -> bool {
        if self.nodes[index_to_process].global_reward.reward_global_with_costs.is_some() {
            return true;
        }
        if indexes_processed.contains(&index_to_process) {
            panic!("There is a cycle in the graph (a nested child has a parent as their child), this means its impossible to compute! Last Node {} Nodes processed(random order): {:#?}\n Graph: {:#?}", self.nodes[index_to_process].original_node_description.clone(), indexes_processed.iter().map(|i| {
                self.nodes[*i].original_node_description.clone()
            }), self);
        }
        indexes_processed.insert(index_to_process);

        // Basically, process the children all completely first, then once they are processed
        // you can then process yourself. And any of your parents can then use your result in their calcs.
        // go through all children and make sure they are calculated first.
        // depth first basically.
        // we process the child TOTALLY. including local, child conns, global total and costs etc.
        for child in self.nodes[index_to_process].children_result.clone() {
            self.calculate_global_reward(og_root_node, map_state, c_state, c_targets, child.child_index, indexes_processed);
        }
        
        

        let target = match &self.nodes[index_to_process].list_target {
            Some(target_id) => Some(target_id.get_node_target(c_targets)),
            None => None,
        };
        // all children must have been processed now.
        // global reward sum is: reward_local + Sum(rewards_per_requirement)
        // final global reward with cost is: global_sum - cost_base / cost_multiplier
        
        let mut conn_by_categories: Vec<Vec<RewardNodeConnection>> = vec![];
        // Remember the indexes of the NodeResults are different than the Nodes because 
        // of the creature list nodes.

        // Ground conns by effect
        self.nodes[index_to_process].children_result.iter().for_each(|conn| {
            let mut existing = false;
            for cvec in conn_by_categories.iter_mut() {
                if cvec.len() > 0 && cvec[0].category == conn.category {
                    existing = true;
                    cvec.push(conn.clone());
                    break;
                }
            }
            if !existing {
                conn_by_categories.push(vec![conn.clone()]);
            }
        });

        let mut conn_results: Vec<BinaryHeap<ConnectionResult>> = Vec::new();
        for conn_list_noderesult in conn_by_categories.iter() {
            let mut cat_results: BinaryHeap<ConnectionResult> = BinaryHeap::new();
            let effect = conn_list_noderesult[0].category;

            if effect == Effect::None {
                // For None category, we SUM all child connections.
                // Since they have no requirements connected to this node, we don't need
                // to do the LIMIT algorithm below or care about counts and other stuff
                let mut total_sum = 0.;
                // if its None, the count is 0, just get the base multiplier in the connection.
                for connection_resultnode in conn_list_noderesult {
                    let base_mult = connection_resultnode.base_multiplier.unwrap();
                    let conn_result = ConnectionResult{
                        category: effect,
                        base_multiplier: base_mult,
                        multiplier_child: 1.,
                        total_reward: vec![get_global_reward_for_connection(1., self.nodes[connection_resultnode.child_index].global_reward.reward_global_with_costs.unwrap(), base_mult)],
                        child_count: 0.,
                        parent_count: 0.,
                        parent_count_total: 0,
                        parent_to_child_count_ratio: 0.,
                        child_index_node_result: connection_resultnode.child_index,
                        parent_index_node_result: connection_resultnode.parent_index,
                    };
                    total_sum+= conn_result.total_reward[0];
                    cat_results.push(conn_result);
                }
                let final_max = EffectReward{
                    reward: total_sum,
                    category: effect,
                };
                self.nodes[index_to_process].global_reward.rewards_per_result_change.push(final_max);
                conn_results.push(cat_results);
                continue;
            }

            // If requirement for child connections are not None Effect then we need
            // to get the MAX connection reward, essentially if doing an action gives some wood and
            // some fiber, that actions reward would be (Best thing to do with wood) + (Best thing to do with fiber)

            // Getting the best thing to do is based on what items we already have
            // to figure this out below is the RESERVE ALGORITHM.
            // Imagine you need Wood as just 1 ingredient to create a bow.
            // If you have enough wood already for 10 bows, then the reward
            // for getting more wood should be reward for making an 11th bow.
            // Now to add to this, imagine we have two recipes requiring wood, bow and shield
            // To get the TRUE reward, we would have to recurssively simulate giving out the wood
            // one by one. So 1 wood to bow because its the highest reward, then the 2nd to 
            // shield. then the 3rd and 4th and 5th to shield because extra shields are more valuable
            // because shields break more often, but then 6th to Bow because we have so many shields.
            // It gets crazy when there are nested rewards, so if Shield also is used to create SpikedShield
            // then we need to recussively check Shield's children based on different numbers of shield we have already.
            // thats computationally expensive and annoying to program so instead we Estimate 
            // the reward of additional items for an item with the reward_connection function
            // which takes in a Count as a parameter.
            let parent_count = get_count_of_effect(map_state, c_state, effect);
            let effect_change = get_effect_change_from_effects(effect, &self.nodes[index_to_process].effects).unwrap().change as f32;
            for connection_node_result in conn_list_noderesult {
                let requirement_needed_in_child = get_smallest_effect_change_from_vec_vec(&self.nodes[connection_node_result.child_index].requirement_result.dynamic_and_static_requirements, effect);
                if requirement_needed_in_child == None {
                    eprintln!("have a effect connection but child requirement does not actually have this Effect as its requirement. Graph is wrong! parent:{} child:{} effect:{:#?}", index_to_process, connection_node_result.child_index, effect);
                    panic!("have a effect connection but child requirement does not have requirement.");
                }
                let conn_requirement_needed = requirement_needed_in_child.unwrap().change as f32;

                // now just need to get "actually exists" of the child. this can be done by
                // for each of its effects, get their count, then for each effect get its 
                // reward proportion by finding its rewards_per_requirement/reward_sum_total
                let child_count = self.nodes[connection_node_result.child_index].get_count(map_state, c_state);
                assert!(child_count >= 0.);

                let base_multiplier = effect_change / conn_requirement_needed;
                assert!(connection_node_result.base_multiplier.is_none()); // only None connections should have a hardCoded base multiplier

                let og_node_idx = self.nodes[connection_node_result.child_index].original_node_index;
                let multiplier_child = (&og_root_node).nodes.get(og_node_idx).unwrap().get_child_multiplier(
                    child_count, 
                    map_state, 
                    c_state, 
                    target.as_ref()
                );
                cat_results.push(ConnectionResult{
                    category: effect,
                    base_multiplier: base_multiplier,
                    multiplier_child: multiplier_child,
                    total_reward: vec![get_global_reward_for_connection(
                        multiplier_child, 
                        self.nodes[connection_node_result.child_index].global_reward.reward_global_with_costs.unwrap(),
                        base_multiplier,
                    )],
                    child_count: child_count,
                    parent_count: 0.,
                    parent_count_total: parent_count,
                    parent_to_child_count_ratio: 1. / conn_requirement_needed,
                    child_index_node_result: connection_node_result.child_index,
                    parent_index_node_result: connection_node_result.parent_index,
                });
            }

            let increment_best_child = |mut top: ConnectionResult| -> ConnectionResult {
                top.parent_count += effect_change * top.parent_to_child_count_ratio;
                top.multiplier_child = (&og_root_node).nodes.get(self.nodes[top.child_index_node_result].original_node_index).unwrap().get_child_multiplier(
                    top.child_count + top.parent_count, 
                    map_state,
                    c_state, 
                    target.as_ref()
                );
                top.total_reward.insert(0, get_global_reward_for_connection(
                    top.multiplier_child, 
                    self.nodes[top.child_index_node_result].global_reward.reward_global_with_costs.unwrap(),
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
            let final_max = EffectReward{
                reward: best.total_reward[0],
                category: best.category,
            };
            cat_results.push(best);
            self.nodes[index_to_process].global_reward.rewards_per_result_change.push(final_max);

            conn_results.push(cat_results);
        }
        self.nodes[index_to_process].connection_results = Some(conn_results);

        let mut reward_sum_total = self.nodes[index_to_process].reward_result.reward_local;

        for effect_result in &self.nodes[index_to_process].global_reward.rewards_per_result_change {
            reward_sum_total += effect_result.reward;
        }
        self.nodes[index_to_process].global_reward.reward_sum_total = Some(reward_sum_total);
        let final_reward = (reward_sum_total - self.nodes[index_to_process].cost_result.cost_base) / self.nodes[index_to_process].cost_result.cost_divider;
        self.nodes[index_to_process].global_reward.reward_global_with_costs = Some(final_reward);
        return true;
    }

    pub fn get_final_command<'l>(&self, og_root_node: &RootNode, map_state: &'l MapState, c_state: &'l CreatureState, c_targets: &'l HashMap<UID, &CreatureState>) -> Option<(CreatureCommand<'l>, String)> {
        // make sure the map is same frame as the result for safety.
        if map_state.frame_count != self.frame {
            panic!("Trying to use map state with different frame than this result node!");
        }
        let mut best_node: Option<&NodeResult> = None;
        for node_result in self.nodes.iter() {
            // Nodes can have None global rewards if they are disconnected from root. 
            // Which can happen when they are meant to connect from dynamic requirement-effect connections and 
            // there are no nodes that have the effect for the requirement.
            if node_result.global_reward.reward_global_with_costs.is_none() {
                assert!(
                    node_result.requirement_result.dynamic_and_static_requirements.iter().map(|r| r.len()).sum::<usize>() > 0
                );
                continue;
            }
            if node_result.global_reward.reward_global_with_costs.unwrap() <= 0. {
                continue;
            }
            if 
                node_result.requirement_result.valid && 
                node_result.has_command && 
                (
                    best_node.is_none()
                    ||
                    node_result.global_reward.reward_global_with_costs.unwrap() >= 
                    best_node.unwrap().global_reward.reward_global_with_costs.unwrap()
                ) {
                    best_node = Some(node_result);
            }
        }
        
        if let Some(best_node) = best_node {
            let original_node = og_root_node.nodes.get(best_node.original_node_index).unwrap();
            let cmd = match original_node {
                Node::Reward(n) => n.get_command.as_ref().unwrap()(map_state, c_state, &best_node.reward_result, &best_node.requirement_result),
                Node::ListNode(nl) => {
                    let target = best_node.list_target.as_ref().unwrap().get_node_target(c_targets);
                    nl.get_command.as_ref().unwrap()(map_state, c_state, &best_node.reward_result, &best_node.requirement_result, target)
                },
            };
            return Some((cmd, best_node.original_node_description.clone()));
        }
        None
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeResult {
    pub index: NodeResultIndex,
    pub original_node_index: NodeIndex,
    pub original_node_description: String,
    pub reward_result: RewardResult, //contains local reward
    pub cost_result: CostResult,
    pub requirement_result: RequirementResult,
    pub effects: Vec<EffectChange>,
    pub children_result: Vec<RewardNodeConnection>,
    // Filled out as you do global reward:
    pub connection_results: Option<Vec<BinaryHeap<ConnectionResult>>>,
    pub global_reward: NodeRewardGlobal,
    pub has_command: bool,
    pub list_target: Option<NodeTargetID>,
}
impl fmt::Debug for NodeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeResult")
            .field("index", &self.index)
            .field("original_node_index", &self.original_node_index)
            .field("original_node_description", &self.original_node_description)
            .field("reward_result", &self.reward_result)
            .field("cost_result", &self.cost_result)
            .field("requirement_result", &self.requirement_result)
            .field("effects", &self.effects)
            .field("children_result", &self.children_result)
            .field("connection_results", &self.connection_results)
            .field("global_reward", &self.global_reward)
            //.field("command_result", &cmd)
            .field("creature_target", &self.list_target)
            .finish()
    }
}

impl NodeResult {
    pub fn get_count(&self, map_state: &MapState, c_state: &CreatureState) -> f32 {
        if self.global_reward.reward_sum_total.is_none() {
            panic!("Trying to get count when global reward has not been calculated yet!");
        }

        // get reward from children and calculate count based on contribution to total reward
        let total_sum = self.global_reward.reward_sum_total.unwrap();
        let mut total_count = 0.;
        for effect_result in &self.global_reward.rewards_per_result_change {
            if effect_result.category != Effect::None {
                total_count += (effect_result.reward/total_sum) * get_count_of_effect(map_state, c_state, effect_result.category) as f32;
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
                let count =  get_count_of_effect(map_state, c_state, effect.effect);
                total_count += count as f32 / total_effect as f32 * local_proportion;
            }
        }

        total_count
    }
}

