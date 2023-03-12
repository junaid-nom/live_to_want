extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::{*, reward_graph::{RootNode, Node, RewardNode, RewardResult, RequirementResult, VariableChange, CostResult, RewardNodeConnection, Variable, ConnectionResult, RewardNodeCreatureList}};

#[test]
fn test_1_tier_reward_graph() {
    // have 3 options. only 2 are possible. 
    // one has higher reward than other.

    let cant_do_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Bone), 
            change: 2 
        }]],
        description: "cant_do bone".to_string(),
        index: 0, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 100.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Bone) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }),
        cost: Box::new(|_, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 5.,
                cost_divider: 5.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("boneseat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let can_do_low_reward = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Meat),
            change: 2 
        }]],
        description: "cant_do meat".to_string(),
        index: 1, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 10.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Meat) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 1 with these costs
            CostResult {
                cost_base: 5.,
                cost_divider: 5.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("meateat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let can_do_high_reward = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Berry), 
            change: 2 
        }]],
        description: "cant_do Berry".to_string(),
        index: 2, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 8.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Berry) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 2 with these costs
            CostResult {
                cost_base: 2.,
                cost_divider: 3.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("berryeat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![cant_do_node, can_do_low_reward, can_do_high_reward],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 1, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 2, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            }
        ],
    };
    let map = MapState::default();
    let creature = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let hash = map.get_creatures_hashmap();
    let result_graph = root.generate_result_graph(&map, &creature, &hash);

    // TODO: og node not set. global rewards not set. move to new file
    println!("{:#?}", result_graph);
    assert_eq!(result_graph.nodes.len(), 3);
    assert_eq!(result_graph.nodes[0].original_node_index, 0);
    assert_eq!(result_graph.nodes[1].original_node_index, 1);
    assert_eq!(result_graph.nodes[2].original_node_index, 2);


    assert_eq!(result_graph.nodes[0].global_reward.reward_global_with_costs.unwrap(), 19.);
    assert_eq!(result_graph.nodes[1].global_reward.reward_global_with_costs.unwrap(), 1.);
    assert_eq!(result_graph.nodes[2].global_reward.reward_global_with_costs.unwrap(), 2.);

    let cmd = result_graph.get_final_command(&root, &map, &creature, &hash);

    match cmd.unwrap().0 {
        CreatureCommand::MoveTo(name, _, _, _) => assert_eq!(name, "berryeat"),
        _ => assert!(false)
    }
}

#[test]
fn test_limit_algo_reward_graph() {
    // have 3 options all require wood + other ingredients
    // spear: req 2 wood: final reward (per wood): 12,9 ,, 6,3 ,, 0..(perwood)
    // shield: req 3. FinalReward: 10, 9, 8 ,, 7, 6, 5 ,, 4 ,, 1, 0..
    // arrow: req 1. finalReward: 9(x),9,9, 1..
    // assume have 1 real arrow already. and 8 wood already (calc 9th)?
    // reward for each wood should be:
    // spear, shield, spear, shield, arrow, arrow, shield, shield, spear, 
    // so final reward should be: spear: 6, shield 6, arrow 1 -> 6

    // NOTE: THERE IS NO MECHANISM to prevent making an item you have requirements for
    // even if the ingredients are better used for a different recipe that is missing something!
    // not an issue if gathering of ingredient has same cost as using it,
    // because then gathering will have higher reward than the not great item.
    // maybe solved by having cost be - reward of all parents? but that's circular.
    // could solve circular problem by making it so its a final "oppertunity cost step"?
    // wait maybe this doesn't matter because child's actual childcount will
    // increase so its actual reward will be low? lower than gathering stuff prob?

    let spear_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Bone), 
                change: 2
            },
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 2
            },
        ]],
        description: "spear".to_string(),
        index: 0, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 24.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, count| {
            1. - (0.5 * count) // 1, .5, 0
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Bone) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("spear", c, Location::new0(), 0))), 
        effect:  Some(Box::new(|_, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Spear), 
                change: 1
            },
        ])),
        }
    );

    let shield_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Skin), 
                change: 2
            },
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 3
            },
        ]],
        description: "shield".to_string(),
        index: 1, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 30.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, count| {
            1. - (0.3 * count) // 1, .7, .4, .1, -.2
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Skin) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("shield", c, Location::new0(), 0))), 
        effect:  Some(Box::new(|_, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Shield), 
                change: 1
            },
        ])),
        }
    );

    let arrow_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Fiber), 
                change: 1
            },
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 1
            },
        ]],
        description: "arrow".to_string(),
        index: 2, 
        static_children: vec![], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 9.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, count| {
            if count < 3. {
                1.
            }
            else {
                1.0/9.0
            }
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Fiber) >= 1,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("arrow", c, Location::new0(), 0))), 
        effect:  Some(Box::new(|_, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Arrow), 
                change: 1
            },
        ])),
        }
    );

    let wood_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![]],
        description: "wood".to_string(),
        index: 3, 
        static_children: vec![
            RewardNodeConnection{
                base_multiplier: None, 
                child_index: 0, 
                parent_index: 3,
                category: Variable::InventoryItem(ItemType::Wood),
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: None, 
                child_index: 1, 
                parent_index: 3,
                category: Variable::InventoryItem(ItemType::Wood),
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: None, 
                child_index: 2, 
                parent_index: 3,
                category: Variable::InventoryItem(ItemType::Wood),
                dont_match_targets: false,
            },

        ],
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 0.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, _| {
            RequirementResult {
                valid: true,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| {
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("wood", c, Location::new0(), 0))), 
        effect:  Some(Box::new(|_, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 1
            },
        ])),
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![spear_node, shield_node, arrow_node, wood_node],
        children: vec![
            RewardNodeConnection{
                base_multiplier: Some(1.), 
                child_index: 3, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
        ],
    };

    let map = MapState::default();
    let creature = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Wood, quantity: 8 },
            Item{ item_type: ItemType::Arrow, quantity: 1 },
        ],
    };
    let hash = map.get_creatures_hashmap();
    let result_graph = root.generate_result_graph(&map, &creature, &hash);

    // check the limit reward thingy for the wood node, and the final rewards for the childen nodes.
    println!("{:#?}", result_graph);
    
    assert_eq!(result_graph.nodes.len(), 4);
    assert_eq!(result_graph.nodes[0].original_node_index, 0);
    assert_eq!(result_graph.nodes[1].original_node_index, 1);
    assert_eq!(result_graph.nodes[2].original_node_index, 2);
    assert_eq!(result_graph.nodes[3].original_node_index, 3);


    // assert_eq!(result_graph.nodes[0].global_reward.reward_global_with_costs.unwrap(), 19.);
    let wood = &result_graph.nodes[3];
    let results: &Option<Vec<std::collections::BinaryHeap<ConnectionResult>>> = &wood.connection_results;
    for conn_result in &results.as_ref().unwrap()[0] {
        if conn_result.child_index_node_result == 0 {
            assert_eq!(conn_result.total_reward, vec![
                6.0,
                9.0,
                12.0,
            ]);
        }
        if conn_result.child_index_node_result == 1 {
            assert_eq!(conn_result.total_reward, vec![
                5.9999995,
                7.0,
                7.9999995,
                9.0,
                10.0,
            ]);
        }
        if conn_result.child_index_node_result == 2 {
            assert_eq!(conn_result.total_reward, vec![
                1.0,
                9.0,
                9.0,
            ]);
        }
    }
    assert_eq!(result_graph.nodes[3].global_reward.reward_global_with_costs.unwrap(), 6.0);

    let cmd = result_graph.get_final_command(&root, &map, &creature, &hash);

    match cmd.unwrap().0 {
        CreatureCommand::MoveTo(name, _, _, _) => assert_eq!(name, "wood"),
        _ => assert!(false)
    }
}


#[test]
fn test_creature_list_node_reward_graph() {
    let openr = RegionCreationStruct::new(9,9, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut creature1 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature2 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
        ],
    };
    let mut creature3 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature4 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature5 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Bone, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };

    creature1.components.location_component.location = Vu2::new(3,4);
    creature2.components.location_component.location = Vu2::new(4,4);
    creature3.components.location_component.location = Vu2::new(4,5);
    creature4.components.location_component.location = Vu2::new(5,4);
    creature5.components.location_component.location = Vu2::new(5,5);


    let _c1_id = creature1.get_id();
    let c2_id = creature2.get_id();
    let c3_id = creature3.get_id();
    let c4_id = creature4.get_id();
    let c5_id = creature5.get_id();

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature2.get_location(), frame_updated: 0, id: c2_id });

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature3.get_location(), frame_updated: 0, id: c3_id });

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature4.get_location(), frame_updated: 0, id: c4_id });
    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature4.get_location(), frame_updated: 0, id: c5_id });

    region.grid[creature1.components.location_component.location].creatures.add_creature(
        creature1, 0
    );
    region.grid[creature2.components.location_component.location].creatures.add_creature(
        creature2, 0
    );
    region.grid[creature3.components.location_component.location].creatures.add_creature(
        creature3, 0
    );
    region.grid[creature4.components.location_component.location].creatures.add_creature(
        creature4, 0
    );
    region.grid[creature5.components.location_component.location].creatures.add_creature(
        creature5, 0
    );

    let inbetween_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Berry), 
            change: 2 
        }]],
        description: "cant_do Berry".to_string(),
        index: 0, 
        static_children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(0.5), 
                child_index: 1, 
                parent_index: 0, 
                category: Variable::None,
                dont_match_targets: false,
            }
        ], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 8.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Berry) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 2 with these costs
            CostResult {
                cost_base: 2.,
                cost_divider: 3.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("berryeat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let list_node = Node::CreatureList(RewardNodeCreatureList {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Fiber), 
                change: 1
            },
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 1
            },
        ]],
        description: "listnode".to_string(),
        index: 1, 
        static_children: vec![],
        reward: Box::new(|_, _, _, other| {
            RewardResult{
                reward_local: other.inventory.len() as f32 * 2.0,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _count, _| {
                1.
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, _c, other| {
            RequirementResult {
                valid: other.get_inventory_of_item(ItemType::Berry) > 0,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_, other| CreatureCommand::MoveTo("a", c, other.get_location(), 0))), 
        effect:  Some(Box::new(|_, _, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Arrow), 
                change: 1
            },
        ])),
        filter: Box::new(|_, c1, other| {
            if other.get_id() == c1.memory.creatures_remembered[0].id {
                return false;
            }
            return true;
        }),
        }
    );
    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![inbetween_node, list_node],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
        ],
    };
    
    let hash = map.get_creatures_hashmap();
    let result_graph = root.generate_result_graph(&map, map.get_creature_list()[0], &hash);

    println!("{:#?}", result_graph);
    println!("2:{} 3:{} 4:{} 5:{}", c2_id, c3_id, c4_id, c5_id);

    // creature 2 is out because its ID is filtered out.
    // creaure 5 has highest reward but no requirements met
    // creature 4 has higher reward than creature 3 so its selected

    let cmd = result_graph.get_final_command(&root, &map, map.get_creature_list()[0], &hash);

    match cmd.unwrap().0 {
        CreatureCommand::MoveTo(_, _, loc, _) => assert_eq!(loc.position, Vu2::new(5,4)),
        _ => assert!(false)
    }
}

// test with root -> creaturelist -> creaturelist
#[test]
fn test_creature_list_node_reward_graph_2layer() {
    let openr = RegionCreationStruct::new(9,9, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut creature1 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature2 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
        ],
    };
    let mut creature3 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature4 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let mut creature5 = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Bone, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };

    creature1.components.location_component.location = Vu2::new(3,4);
    creature2.components.location_component.location = Vu2::new(4,4);
    creature3.components.location_component.location = Vu2::new(4,5);
    creature4.components.location_component.location = Vu2::new(5,4);
    creature5.components.location_component.location = Vu2::new(5,5);


    let _c1_id = creature1.get_id();
    let c2_id = creature2.get_id();
    let c3_id = creature3.get_id();
    let c4_id = creature4.get_id();
    let c5_id = creature5.get_id();

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature2.get_location(), frame_updated: 0, id: c2_id });

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature3.get_location(), frame_updated: 0, id: c3_id });

    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature4.get_location(), frame_updated: 0, id: c4_id });
    creature1.memory.creatures_remembered.push(CreatureRemembered { location: creature4.get_location(), frame_updated: 0, id: c5_id });

    region.grid[creature1.components.location_component.location].creatures.add_creature(
        creature1, 0
    );
    region.grid[creature2.components.location_component.location].creatures.add_creature(
        creature2, 0
    );
    region.grid[creature3.components.location_component.location].creatures.add_creature(
        creature3, 0
    );
    region.grid[creature4.components.location_component.location].creatures.add_creature(
        creature4, 0
    );
    region.grid[creature5.components.location_component.location].creatures.add_creature(
        creature5, 0
    );

    let inbetween_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Berry), 
            change: 2 
        }]],
        description: "inbetween".to_string(),
        index: 0, 
        static_children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(0.5), 
                child_index: 1, 
                parent_index: 0, 
                category: Variable::None,
                dont_match_targets: false,
            }
        ], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 8.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Berry) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 2 with these costs
            CostResult {
                cost_base: 2.,
                cost_divider: 3.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("berryeat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let list_node = Node::CreatureList(RewardNodeCreatureList {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Fiber), 
                change: 1
            },
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Wood), 
                change: 1
            },
        ]],
        description: "listnode_1".to_string(),
        index: 1, 
        static_children: vec![],
        reward: Box::new(|_, _, _, other| {
            RewardResult{
                reward_local: other.inventory.len() as f32 * 1.0,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _count, _| {
                1.
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, _c, other| {
            RequirementResult {
                valid: other.get_inventory_of_item(ItemType::Berry) > 0,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_, other| {
            let loca = other.get_location();
            CreatureCommand::MoveTo("a", c, loca, 0)
        }
        )), 
        effect:  Some(Box::new(|_, _, _, _, _| vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Arrow), 
                change: 1
            },
        ])),
        filter: Box::new(|_, c1, other| {
            if other.get_id() == c1.memory.creatures_remembered[0].id {
                return false;
            }
            return true;
        }),
        }
    );

    // Should auto child to listnode1 from requirements/effect pairings
    let list_node_2 = Node::CreatureList(RewardNodeCreatureList {
        static_requirements: vec![vec![
            VariableChange{ 
                variable: reward_graph::Variable::InventoryItem(ItemType::Arrow), 
                change: 1
            },
        ]],
        description: "listnode_2".to_string(),
        index: 2, 
        static_children: vec![],
        reward: Box::new(|_, _, _, other| {
            RewardResult{
                reward_local: other.inventory.len() as f32 * 100.0,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _count, _| {
                1.
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, _c, _other| {
            RequirementResult {
                valid: false,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }),
        cost: Box::new(|_, _, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_, other| {
            let mut loca = other.get_location();
            loca.position = Vu2::new(999, 999);
            CreatureCommand::MoveTo("a", c, loca, 0)
        }
        )),  
        effect: None,
        filter: Box::new(|_, c1, other| {
            if other.get_id() == c1.memory.creatures_remembered[0].id {
                return false;
            }
            return true;
        }),
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![inbetween_node, list_node, list_node_2],
        children: vec![
            RewardNodeConnection{
                base_multiplier: Some(1.), 
                child_index: 0,
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
        ],
    };
    
    let hash = map.get_creatures_hashmap();
    let result_graph = root.generate_result_graph(&map, map.get_creature_list()[0], &hash);

    println!("{:#?}", result_graph);
    println!("2:{} 3:{} 4:{} 5:{}", c2_id, c3_id, c4_id, c5_id);

    // creature 2 is out because its ID is filtered out.
    // creaure 5 has highest reward but no requirements met
    // creature 4 has higher reward than creature 3 so its selected

    for node in &result_graph.nodes {
        assert!(node.global_reward.reward_global_with_costs.is_some());
        assert!(node.global_reward.reward_global_with_costs.unwrap() > 0.);
    }
    let cmd = result_graph.get_final_command(&root, &map, map.get_creature_list()[0], &hash);

    match cmd.unwrap().0 {
        CreatureCommand::MoveTo(_, _, loc, _) => assert_eq!(loc.position, Vu2::new(5,4)),
        _ => assert!(false)
    }
}

#[test]
#[should_panic]
fn test_loop_in_reward_graph() {
    // make sure this fails
    let cant_do_node = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Bone), 
            change: 2 
        }]],
        description: "cant_do bone".to_string(),
        index: 0, 
        static_children: vec![
            RewardNodeConnection { 
                base_multiplier: Some(1.), 
                child_index: 1, 
                parent_index: 0, 
                category: Variable::None,
                dont_match_targets: false,
            }
        ], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 100.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Bone) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }),
        cost: Box::new(|_, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 5.,
                cost_divider: 5.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("boneseat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let can_do_low_reward = Node::Reward(RewardNode {
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::Meat), 
            change: 2 
        }]],
        description: "cant_do meat".to_string(),
        index: 1, 
        static_children: vec![
            RewardNodeConnection { 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 1, 
                category: Variable::None,
                dont_match_targets: false,
            }
        ], 
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 10.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }), 
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::Meat) >= 2,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }), 
        cost: Box::new(|_, _, _| { // total reward should be 1 with these costs
            CostResult {
                cost_base: 5.,
                cost_divider: 5.,
            }
        }), 
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::MoveTo("meateat", c, Location::new0(), 0))), 
        effect: None,
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![cant_do_node, can_do_low_reward],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 1, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 2, 
                parent_index: 0,
                category: Variable::None,
                dont_match_targets: false,
            }
        ],
    };
    let map = MapState::default();
    let creature = CreatureState {
        components: ComponentMap::default(),
        memory: CreatureMemory { creatures_remembered: vec![] },
        inventory: vec![
            Item{ item_type: ItemType::Berry, quantity: 2 },
            Item{ item_type: ItemType::Meat, quantity: 2 },
        ],
    };
    let hash = map.get_creatures_hashmap();
    let _result_graph = root.generate_result_graph(&map, &creature, &hash); // will fail
}
