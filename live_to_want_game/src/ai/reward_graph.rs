use serde::{Deserialize, Serialize};

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

pub fn get_global_reward_for_connection(node: &Node, global_reward: f32, counts: &Vec<i32>, m: &MapState, c: &CreatureState, c_target: Option<&CreatureState>) -> f32 {
    match node {
        Node::Reward(n) => n.reward_connection.as_ref()(m, c, counts) * global_reward,
        Node::CreatureList(nl) => nl.reward_connection.as_ref()(m, c, counts, c_target.unwrap()) * global_reward,
    }
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

                    // If there is an inbetween material to make something, and we have enough to make it already.
                    // then we should add 1 to the actual count of the child object, because we already have enough to make one more of it for this ingredient.
                    // For example if it takes 5 wood to make a plank, and 1 plank and 1 stone to make a spear,
                    // then if you already have 5 wood, the reward of gathering more wood should be as if we already 
                    // have +1 spear than we have because we already have wood to make +1 spear.
                    let mut this_counts: Vec<i32> = effects.iter().map(|e| get_count_of_variable(map_state, c_state, e.variable)).collect();

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
                            if cvec.len() > 0 && cvec[0].requirement == conn.requirement {
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
                    let counts_default = vec![];
                    for conn_list in conn_by_categories.iter() {
                        let mut cat_results: Vec<ConnectionResult> = vec![];
                        for c in conn_list {
                            cat_results.push(ConnectionResult{
                                multiplier: c.base_multiplier,
                                global_reward: Some(
                                    get_global_reward_for_connection(
                                        (&self).nodes.get((*c).child_index).unwrap(), 
                                        root.nodes[(*c).child_index].global_reward.unwrap(), 
                                        &counts_default, 
                                        map_state, 
                                        c_state,
                                        None)
                                    ),
                                bonus_count: 0,
                            });
                        }
                        let count = get_count_of_variable(map_state, c_state, conn_list[0].requirement);
                        for i in 0..count {
                            // get the Max reward of them all, increase its count by 1 and recompute it.
                        }
                    }

                    let mut new = NodeResult{
                        requirement_result: requirement,
                        reward_result: reward,
                        cost_result: cost,
                        global_reward: None,
                        children: (&n.children).into_iter().map(|c| c.child_index).collect(),
                        effects: effects,
                        connection_results: (&n.children).into_iter().map(|c| {
                            // Get if requirements met.
                            match self.nodes.get(c.child_index).unwrap() {
                                Node::Reward(r) => r.reward_connection.as_ref()(map_state, c_state, &this_counts),
                                Node::CreatureList(_) => todo!(),
                            }
                        }).collect(), // need to wait for global results of children to compute this
                        original_node: n.index,
                    };
                    root.nodes.push(new);
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
}
pub struct VariableChange {
    pub variable: Variable,
    pub change: i32,
}

pub struct RewardNodeConnection {
    pub base_multiplier: f32, // for example if 5 wood is needed for a spear, the multiplier should be 1/5
    pub child_index: NodeIndex,
    pub parent_index: NodeIndex,
    pub requirement: Variable,
}
pub struct RewardNode {
    pub description: String,  // just for debugging/comments
    pub index: NodeIndex,
    pub children: Vec<RewardNodeConnection>,
    //pub parents: Vec<NodeIndex>,
    pub reward: Box<fn(&MapState, &CreatureState, &RequirementResult) -> RewardResult>,
    pub reward_connection: Box<fn(&MapState, &CreatureState, &Vec<i32>) -> f32>,
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
    pub reward_connection: Box<fn(&MapState, &CreatureState, &Vec<i32>, &CreatureState) -> f32>,
    pub requirement: Box<fn(&MapState, &CreatureState, &CreatureState) -> RequirementResult>,
    pub cost: Box<fn(&MapState, &CreatureState, &RequirementResult, &CreatureState) -> CostResult>,
    pub get_command: Option<Box<for<'f> fn(&'f MapState, &'f CreatureState, &RewardResult, &RequirementResult, &'f CreatureState) -> CreatureCommand<'f>>>, // Is None if this node does not lead to a category and is more of an organizing node
    pub effect: Option<Box<fn(&MapState, &CreatureState, &RewardResult, &RequirementResult, &CreatureState) -> Vec<VariableChange>>>, // Used to get current of self already
    pub filter: Box<fn(&MapState, &CreatureState, &CreatureState)->bool>, // will take all known CreatureStates, then use this filter on them, to produce one NodeResult for each one.
}

pub struct RequirementResult{
    pub valid: bool,
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
pub struct RewardResult{
    pub base_reward: f32,
    // below can be used by other functions to do interesting stuff
    pub target_id: Option<UID>,
    pub target_location: Option<Location>,
}
pub struct CostResult{
    pub cost_base: f32,
    pub cost_divider: f32,
}
pub struct ConnectionResult {
    pub multiplier: f32, // base multiplier * child's Count based multiplier
    pub global_reward: Option<f32>, // None when not set yet
    pub bonus_count: i32, // used to compute the final reward multiplier for the child for his connection
}

pub struct NodeResultRoot {
    pub nodes: Vec<NodeResult>,
    pub children: Vec<NodeIndex>,
    pub original_node_descriptor: String,
}
pub struct NodeResult {
    pub reward_result: RewardResult,
    pub cost_result: CostResult,
    pub requirement_result: RequirementResult,
    pub global_reward: Option<f32>, // none is not calculated yet
    pub children: Vec<NodeIndex>,
    pub effects: Vec<VariableChange>,
    pub connection_results: Vec<ConnectionResult>, // indexs should match children
    pub original_node: NodeIndex,
}
