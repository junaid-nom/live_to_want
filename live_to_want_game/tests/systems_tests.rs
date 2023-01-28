extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::{*, reward_graph::{RootNode, Node, RewardNode, RewardResult, RequirementResult, VariableChange, CostResult, RewardNodeConnection, Variable, ConnectionResult, RewardNodeCreatureList}};

#[test]
fn run_frames_test_starvation_and_death() {
    let root_goal = generate_goal_nodes();

    // create initial mapstate
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    //make creature
    let start_loc = Location::new(Vu2::new(0,0), Vu2::new(1,1));
    let mut c = CreatureState::new_location(start_loc);
    
    c.components.health_component = Some(HealthComponent{
        health: 10,
        max_health: 10,
    });
    c.components.starvation_component = Some(StarvationComponent{
        calories: 1000,
        metabolism: 100,
    });
    c.components.death_items_component = Some(DeathItemsComponent{
        items_to_drop: vec![Item::new(ItemType::Bone, 7)],
    });
    c.inventory.push(Item::new(ItemType::Meat, 6));
    
    println!("Creature id: {}", c.components.id_component.id());

    map.regions[start_loc].creatures.add_creature(c, 0);

    let mut gs = GameState{map_state:map};

    println!("creatures at target: {:#?}", gs.map_state.regions[start_loc].creatures);
    for f in 0..20 {
        println!("running {}", f);
        gs = run_frame(gs, &root_goal);
        println!("creatures at target: {:#?}", gs.map_state.regions[start_loc].creatures);
    }
    println!("items at target: {:#?}", gs.map_state.regions[start_loc].items);
    assert_eq!(gs.map_state.regions[start_loc].creatures.get_length(), Some(0));
    assert_eq!(vec![
        Item {
            item_type: ItemType::Bone,
            quantity: 7,
        },
        Item {
            item_type: ItemType::Meat,
            quantity: 6,
        },
    ], gs.map_state.regions[start_loc].items);
}

// tests for metabolism that checks to see if traits and if moving stuff works.
// Prob can just postpone for awhile and do 1 test that uses EVERY trait that changes them and make 1 big calculation.
// test: 
// traits that influence metabolism:
// starving (already losing health from hunger) STARVING_SLOW_METABOLISM_FACTOR
// moving: MOVING_INCREASED_METABOLISM_FACTOR: traits.traits.move_speed as f32 * MOVE_SPEED_METABOLISM_MULTIPLIER;
// pregnant: STANDARD_PREGNANCY_METABOLISM_MULTIPLIER * LITTER_SIZE_METABOLISM_MULTIPLIER * traits.traits.litter_size
// is child: adult_percent
//   - fast grower: traits.traits.fast_grower as f32 * FAST_GROWER_CALORIE_MULTIPLIER
// thick_hide: traits.traits.thick_hide as f32 * THICK_HIDE_METABOLISM_MULTIPLIER
// So two tests, one is: pregnant + moving + thick_hide. Other is: child + staying still + thick hide?
// maybe also mini test for is starvig?
#[test]
fn test_metabolism_basic<'a>() {
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 1000, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: 100,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: false,
    });
    deer1.setup_creature(1, false);// must use frame become adult if you want adult or assert fails
    starvation_system(&mut deer1, 10);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, 900);
}

#[test]
fn starvation_starving() {
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 0, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            litter_size: 50,
            pregnancy_time: 100,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: false,
    });
    deer1.setup_creature(1, false); // must use frame become adult
    starvation_system(&mut deer1, 10);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, 0 - (100. * STARVING_SLOW_METABOLISM_FACTOR) as i32);
}

#[test]
fn starvation_pregnant() {
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 1000, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            litter_size: 50,
            pregnancy_time: 100,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: false,
    });
    deer1.components.sexual_reproduction = Some(SexualReproduction { is_pregnant: true, pregnancy_completion_frame: 200, litter_size: 1, partner_genes: EvolvingTraits::default() });
    deer1.setup_creature(1, false); // must use frame become adult
    starvation_system(&mut deer1, 10);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, (1000. - (100. * STANDARD_PREGNANCY_METABOLISM_MULTIPLIER * 1.5)) as i32 ); // 1.5 is: 1/100 * 50, the litter_size formula
}
#[test]
fn starvation_moving() {
    // Should test moving is true and also the move_speed trait
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 1000, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            move_speed: 60,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: true,
    });
    deer1.setup_creature(1, false);// must use frame become adult
    starvation_system(&mut deer1, 10);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, (1000. - (100. * MOVING_INCREASED_METABOLISM_FACTOR * (0.3 * 60.))) as i32);
}
#[test]
fn starvation_child() {
    // test ur a child but also fast_grower
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 1000, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            fast_grower: 100,
            ..Default::default()
        },
        born_on_frame: 0,
        child_until_frame: 10,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: false,
    });
    deer1.setup_creature(2, false); // 1/5 way to adult
    starvation_system(&mut deer1, 2);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, 1000 - (100. * 1.5 * 0.2) as i32); // 1.5 because fast grower, .2 because child
}
#[test]
fn starvation_thick_hide() {
    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.starvation_component = Some(StarvationComponent { 
        calories: 1000, 
        metabolism: 100,
    });
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            thick_hide: 200,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: 5,
        destination: Location { region: Vu2{x: 1, y: 1}, position: Vu2{x: 1, y: 1}, },
        frame_ready_to_move: 5,
        moving: false,
    });
    deer1.setup_creature(1, false);// must use frame become adult if you want adult or assert fails
    starvation_system(&mut deer1, 10);
    let calories = deer1.components.starvation_component.unwrap().calories;
    println!("Calories: {}", calories);
    assert_eq!(calories, (1000. - 100. * 1.4) as i32); //thick hide is 1+ .2 per 100 so 1.4 with 200
}

#[test]
fn vision_system_test() {
    let openr = RegionCreationStruct::new(9,9, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer1.components.location_component = LocationComponent {
        location: Vu2{x:1, y: 1}
    };
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: 100,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.vision_component = Some(VisionComponent { visible_creatures: vec![] });

    deer1.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut deer2 =CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer2.components.location_component = LocationComponent {
        location: Vu2{x: 6, y: 1}
    };
    deer2.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer2.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            thick_hide: 200,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: STANDARD_CHILD_TIME as i32, // Should be 0 as child 
            maleness: 100,
            fast_grower: 50,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer2.components.vision_component = Some(VisionComponent { visible_creatures: vec![] });

    let mut deer3 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer3.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer3.components.location_component = LocationComponent {
        location: Vu2{x: 6, y: 2}
    };
    deer3.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    deer3.components.battle_component = Some(BattleComponent {
        in_battle: None,
    });
    deer3.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            move_speed: 200,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 2,
            pregnancy_time: -200,
            cannibal_childbirth: (STANDARD_PREGNANCY_LIVE_WEIGHT as f32 * CANNIBAL_PREGNANCY_DEATH_WEIGHT_MULTIPLIER) as i32,
            maleness: 0,
            fast_grower: 0,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer3.components.vision_component = Some(VisionComponent { visible_creatures: vec![] });

    let deer1_id = deer1.components.id_component.id();
    let deer2_id = deer2.components.id_component.id();
    let deer3_id = deer3.components.id_component.id();

    region.grid[deer1.components.location_component.location].creatures.add_creature(
        deer1, 0
    );
    region.grid[deer2.components.location_component.location].creatures.add_creature(
        deer2, 0
    );
    region.grid[deer3.components.location_component.location].creatures.add_creature(
        deer3, 0
    );
    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 3);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);
    println!("creatures: {}", game_state.map_state.get_creature_strings());

    let goal_node = GoalNode::generate_single_node_graph();

    for _ in 0..1 {
        game_state = run_frame(game_state, &goal_node);
        println!("creatures: {}", game_state.map_state.get_creature_strings());
    }
    assert_eq!(game_state.map_state.get_creature_list().len(), 3);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);

    game_state.map_state.get_creature_list().iter().for_each(|c| {
        let id = c.get_id();
        if id == deer1_id {
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures.len() == 1);
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures[0] == deer2_id);
        }
        if id == deer2_id {
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures.len() == 2);
            // too male to become pregnant (unless another super male was added)
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures.contains(&deer1_id));
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures.contains(&deer3_id));
        }
        if id == deer3_id {
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures.len() == 1);
            assert!(c.components.vision_component.as_ref().unwrap().visible_creatures[0] == deer2_id);
        }
    });
}

#[test]
fn test_1_tier_reward_graph() {
    // have 3 options. only 2 are possible. 
    // one has higher reward than other.

    let cant_do_node = Node::Reward(RewardNode { 
        description: "cant_do bone".to_string(),
        index: 0, 
        children: vec![], 
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
                requirements: vec![vec![VariableChange{ 
                    variable: reward_graph::Variable::Bone, 
                    change: 2 
                }]],
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
        description: "cant_do meat".to_string(),
        index: 1, 
        children: vec![], 
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
                requirements: vec![vec![VariableChange{ 
                    variable: reward_graph::Variable::Meat, 
                    change: 2 
                }]],
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
        description: "cant_do Berry".to_string(),
        index: 2, 
        children: vec![], 
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
                requirements: vec![vec![VariableChange{ 
                    variable: reward_graph::Variable::Berry, 
                    change: 2 
                }]],
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
                requirement: VariableChange { variable: Variable::None, change: 0 } 
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                requirement: VariableChange { variable: Variable::None, change: 0 } 
            },
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                requirement: VariableChange { variable: Variable::None, change: 0 } 
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
    let result_graph = root.generate_result_graph(&map, &creature);

    // TODO: og node not set. global rewards not set. move to new file
    println!("{:#?}", result_graph);
    assert_eq!(result_graph.nodes.len(), 3);
    assert_eq!(result_graph.nodes[0].original_node, 0);
    assert_eq!(result_graph.nodes[1].original_node, 1);
    assert_eq!(result_graph.nodes[2].original_node, 2);


    assert_eq!(result_graph.nodes[0].global_reward.reward_global_with_costs.unwrap(), 19.);
    assert_eq!(result_graph.nodes[1].global_reward.reward_global_with_costs.unwrap(), 1.);
    assert_eq!(result_graph.nodes[2].global_reward.reward_global_with_costs.unwrap(), 2.);

    let cmd = result_graph.get_final_command();

    match cmd.unwrap() {
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
        description: "spear".to_string(),
        index: 0, 
        children: vec![], 
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
                requirements: vec![vec![
                    VariableChange{ 
                        variable: reward_graph::Variable::Bone, 
                        change: 2
                    },
                    VariableChange{ 
                        variable: reward_graph::Variable::Wood, 
                        change: 2
                    },
                ]],
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
                variable: reward_graph::Variable::Spear, 
                change: 1
            },
        ])),
        }
    );

    let shield_node = Node::Reward(RewardNode { 
        description: "shield".to_string(),
        index: 1, 
        children: vec![], 
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
                requirements: vec![vec![
                    VariableChange{ 
                        variable: reward_graph::Variable::Skin, 
                        change: 2
                    },
                    VariableChange{ 
                        variable: reward_graph::Variable::Wood, 
                        change: 3
                    },
                ]],
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
                variable: reward_graph::Variable::Shield, 
                change: 1
            },
        ])),
        }
    );

    let arrow_node = Node::Reward(RewardNode { 
        description: "arrow".to_string(),
        index: 2, 
        children: vec![], 
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
                requirements: vec![vec![
                    VariableChange{ 
                        variable: reward_graph::Variable::Fiber, 
                        change: 1
                    },
                    VariableChange{ 
                        variable: reward_graph::Variable::Wood, 
                        change: 1
                    },
                ]],
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
                variable: reward_graph::Variable::Arrow, 
                change: 1
            },
        ])),
        }
    );

    let wood_node = Node::Reward(RewardNode { 
        description: "wood".to_string(),
        index: 3, 
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: None, 
                child_index: 0, 
                parent_index: 3,
                requirement: VariableChange { variable: Variable::Wood, change: 2 } 
            },
            RewardNodeConnection{ 
                base_multiplier: None, 
                child_index: 1, 
                parent_index: 3,
                requirement: VariableChange { variable: Variable::Wood, change: 3 } 
            },
            RewardNodeConnection{ 
                base_multiplier: None, 
                child_index: 2, 
                parent_index: 3,
                requirement: VariableChange { variable: Variable::Wood, change: 1 } 
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
                requirements: vec![vec![
                ]],
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
                variable: reward_graph::Variable::Wood, 
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
                requirement: VariableChange { variable: Variable::None, change: 0 } 
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
    let result_graph = root.generate_result_graph(&map, &creature);

    // check the limit reward thingy for the wood node, and the final rewards for the childen nodes.
    println!("{:#?}", result_graph);
    
    assert_eq!(result_graph.nodes.len(), 4);
    assert_eq!(result_graph.nodes[0].original_node, 0);
    assert_eq!(result_graph.nodes[1].original_node, 1);
    assert_eq!(result_graph.nodes[2].original_node, 2);
    assert_eq!(result_graph.nodes[3].original_node, 3);


    // assert_eq!(result_graph.nodes[0].global_reward.reward_global_with_costs.unwrap(), 19.);
    let wood = &result_graph.nodes[3];
    let results: &Option<Vec<std::collections::BinaryHeap<ConnectionResult>>> = &wood.connection_results;
    for conn_result in &results.as_ref().unwrap()[0] {
        if conn_result.child_index == 0 {
            assert_eq!(conn_result.total_reward, vec![
                6.0,
                9.0,
                12.0,
            ]);
        }
        if conn_result.child_index == 1 {
            assert_eq!(conn_result.total_reward, vec![
                5.9999995,
                7.0,
                7.9999995,
                9.0,
                10.0,
            ]);
        }
        if conn_result.child_index == 2 {
            assert_eq!(conn_result.total_reward, vec![
                1.0,
                9.0,
                9.0,
            ]);
        }
    }
    assert_eq!(result_graph.nodes[3].global_reward.reward_global_with_costs.unwrap(), 6.0);

    let cmd = result_graph.get_final_command();

    match cmd.unwrap() {
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


    let c1_id = creature1.get_id();
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

    let list_node = Node::CreatureList(RewardNodeCreatureList {
        description: "listnode".to_string(),
        index: 0, 
        children: vec![], 
        reward: Box::new(|_, _, _, other| {
            RewardResult{
                reward_local: other.inventory.len() as f32 * 2.0,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, count, _| {
                1.
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, c, other| {
            RequirementResult {
                valid: other.get_inventory_of_item(ItemType::Berry) > 0,
                requirements: vec![vec![
                    VariableChange{ 
                        variable: reward_graph::Variable::Fiber, 
                        change: 1
                    },
                    VariableChange{ 
                        variable: reward_graph::Variable::Wood, 
                        change: 1
                    },
                ]],
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
                variable: reward_graph::Variable::Arrow, 
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
        nodes: vec![list_node],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                requirement: VariableChange { variable: Variable::None, change: 0 } 
            },
        ],
    };

    let result_graph = root.generate_result_graph(&map, map.get_creature_list()[0]);

    println!("{:#?}", result_graph);
    println!("2:{} 3:{} 4:{} 5:{}", c2_id, c3_id, c4_id, c5_id);

    // creature 2 is out because its ID is filtered out.
    // creaure 5 has highest reward but no requirements met
    // creature 4 has higher reward than creature 3 so its selected

    let cmd = result_graph.get_final_command();

    match cmd.unwrap() {
        CreatureCommand::MoveTo(_, _, loc, _) => assert_eq!(loc.position, Vu2::new(5,4)),
        _ => assert!(false)
    }
}

#[test]
fn test_loop_in_reward_graph() {
    // make sure this fails
}


// Test sex, and then reproduction. Make sure the sex related stuff like species, multithreads, mutating, inheritance, litter size, pregnancy time, and childness work.
#[test]
fn test_sex_reproduction<'a>() {
    // make a mapstate with some deer
    let openr = RegionCreationStruct::new(9,9, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer1.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: 100,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer1.components.sexual_reproduction = Some(SexualReproduction {
        is_pregnant: false,
        pregnancy_completion_frame: 1,
        litter_size: 1,
        partner_genes: EvolvingTraits{ ..Default::default() },
    });

    deer1.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut deer2 =CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer2.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    deer2.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer2.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            thick_hide: 200,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: STANDARD_CHILD_TIME as i32, // Should be 0 as child 
            maleness: 100,
            fast_grower: 50,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer2.components.sexual_reproduction = Some(SexualReproduction {
        is_pregnant: false,
        pregnancy_completion_frame: 1,
        litter_size: 1,
        partner_genes: EvolvingTraits{ ..Default::default() },
    });


    let mut deer3 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer3.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer3.components.location_component = LocationComponent {
        location: Vu2{x: 2, y: 1}
    };
    deer3.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    deer3.components.battle_component = Some(BattleComponent {
        in_battle: None,
    });
    deer3.components.evolving_traits = Some(EvolvingTraitsComponent {
        adult_traits: EvolvingTraits{
            species: 0,
            move_speed: 200,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 2,
            pregnancy_time: -200,
            cannibal_childbirth: (STANDARD_PREGNANCY_LIVE_WEIGHT as f32 * CANNIBAL_PREGNANCY_DEATH_WEIGHT_MULTIPLIER) as i32,
            maleness: 0,
            fast_grower: 0,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer3.components.sexual_reproduction = Some(SexualReproduction {
        is_pregnant: false,
        pregnancy_completion_frame: 1,
        litter_size: 1,
        partner_genes: EvolvingTraits{ ..Default::default() },
    });
    

    let mut deer4 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer4.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer4.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 2}
    };
    deer4.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    deer4.components.battle_component = Some(BattleComponent {
        in_battle: None,
    });
    deer4.components.evolving_traits = Some(EvolvingTraitsComponent { // can't mate
        adult_traits: EvolvingTraits{
            species: SPECIES_SEX_RANGE + 1,
            sharp_claws: 200,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: STANDARD_PREGNANCY_TIME as i32 * 2,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer4.components.sexual_reproduction = Some(SexualReproduction {
        is_pregnant: false,
        pregnancy_completion_frame: 1,
        litter_size: 1,
        partner_genes: EvolvingTraits{ ..Default::default() },
    });


    let mut deer5 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer5.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer5.components.location_component = LocationComponent {
        location: Vu2{x: 2 + MAX_ATTACK_DISTANCE, y: 2 + MAX_ATTACK_DISTANCE}
    };
    deer5.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    deer5.components.battle_component = Some(BattleComponent {
        in_battle: None,
    });
    deer5.components.evolving_traits = Some(EvolvingTraitsComponent { // can't mate
        adult_traits: EvolvingTraits{
            species: 0,
            sharp_claws: 777,
            litter_size: LITTER_SIZE_TRAIT_NEEDED_FOR_ONE_BABY * 3 + 50,
            pregnancy_time: STANDARD_PREGNANCY_TIME as i32 * 2,
            maleness: 0,
            fast_grower: 100,
            ..Default::default()
        },
        child_until_frame: 1,
        ..Default::default()
    });
    deer5.components.sexual_reproduction = Some(SexualReproduction {
        is_pregnant: false,
        pregnancy_completion_frame: 1,
        litter_size: 1,
        partner_genes: EvolvingTraits{ ..Default::default() },
    });

    println!("pregnancy time: {} child time: {}", STANDARD_PREGNANCY_TIME, STANDARD_CHILD_TIME);

    println!("deer1 {}", deer1);
    println!("deer2 {}", deer2);
    println!("deer3 {}", deer3);
    println!("deer4 {}", deer4);

    
    let deer1_id = deer1.components.id_component.id();
    let deer2_id = deer2.components.id_component.id();
    let deer3_id = deer3.components.id_component.id();
    let deer4_id = deer4.components.id_component.id();
    let deer5_id = deer5.components.id_component.id();

    region.grid[deer1.components.location_component.location].creatures.add_creature(
        deer1, 0
    );
    region.grid[deer2.components.location_component.location].creatures.add_creature(
        deer2, 0
    );
    region.grid[deer3.components.location_component.location].creatures.add_creature(
        deer3, 0
    );
    region.grid[deer4.components.location_component.location].creatures.add_creature(
        deer4, 0
    );
    region.grid[deer5.components.location_component.location].creatures.add_creature(
        deer5, 0
    );
    
    let attack = GoalNode {
        get_want_local: Box::new(|_, _| 10),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "sex",
        get_command: Some(Box::new(|m: & MapState, c| {
            let mate_cmd = m.get_creature_list().iter().find(|c2| {
                return c.can_sex(c2.get_id(), c2.components.evolving_traits.as_ref().unwrap().adult_traits.species, c2.get_location(), m.frame_count)
                && c2.can_sex(c.get_id(), c.components.evolving_traits.as_ref().unwrap().adult_traits.species, c.get_location(), m.frame_count);
            }).map(|c2| {
                return CreatureCommand::Sex(
                    "sex_deer_closest_can", 
                    c,
                    c2,
                    m.frame_count);
            });
            if mate_cmd.is_some() {
                return mate_cmd.unwrap();
            }

            let target =m.find_closest_creature_to_creature(c).unwrap();
            let cmd = CreatureCommand::Sex(
                "sex_deer_closest", 
                c, 
                target,
                m.frame_count);
            //println!("Choosing for {} target: {}", c.get_id(), target.get_id());
            return cmd;
        })),
        get_requirements_met: Box::new(|m, c| m.find_closest_creature_to_creature(c).is_some()),
    };
    //let root = GoalNode::generate_single_node_graph(attack);

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 5);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);
    assert_eq!(game_state.map_state.get_creature_item_list()[0].1, deer1_id);

    println!("creatures: {}", game_state.map_state.get_creature_strings());

    for _ in 0..7 {
        game_state = run_frame(game_state, &attack);
        println!("creatures: {}", game_state.map_state.get_creature_strings());
    }

    // no one dead
    assert_eq!(game_state.map_state.get_creature_list().len(), 7); // 1 mom dies, 3 kids born
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);

    game_state.map_state.get_creature_list().iter().for_each(|c| {
        let id = c.get_id();
        if id == deer1_id {
            assert!(c.components.sexual_reproduction.as_ref().unwrap().is_pregnant)
        }
        if id == deer2_id {
            // too male to become pregnant (unless another super male was added)
            assert!(!c.components.sexual_reproduction.as_ref().unwrap().is_pregnant)
        }
        if id == deer3_id {
            assert!(c.components.sexual_reproduction.as_ref().unwrap().is_pregnant)
        }
        if id == deer4_id {
            // not same species so no mate.
            assert!(!c.components.sexual_reproduction.as_ref().unwrap().is_pregnant)
        }
        if id == deer5_id {
            // not nearby so no mate.
            assert!(!c.components.sexual_reproduction.as_ref().unwrap().is_pregnant)
        }
    });

    // actually test the reproduction part, see if kids come out at the right times.
    for x in 7..(STANDARD_PREGNANCY_TIME as f32 * 3.0) as i32 {
        if x == STANDARD_PREGNANCY_TIME as i32 * 2 {
            println!("creatures: {}", game_state.map_state.get_creature_strings());
        }
        game_state = run_frame(game_state, &attack);
    }
    println!("creatures: {}", game_state.map_state.get_creature_strings());

    let mut total_adults = 0;
    // Check if kids become adults.
    game_state.map_state.get_creature_list().iter().for_each(|c| {
        if !c.get_if_child(game_state.map_state.frame_count) {
            total_adults += 1;
        }
    });

    assert!(total_adults >= 7); // deer1 should produce at least 3 kids and they shud be mature now
}

// Make a test for simple attack system. Prob similar to the test_chain_multithread_battle test
// Shud use for example thickness and sharp claws. can enhance later with all the other traits
#[test]
// create a map. have two deer. have two deer at the same time declare attack on each other.
// then also check to make sure the battle actually finishes with the expected result: one deer dead, the other with the first deers items
fn test_simple_attack<'a>() {
    //let x: Vec<u32> = (0..100).collect();
    //let y: i32 = x.into_par_iter().map(|_| {}).sum();
    //assert_eq!(y, 100);

    // make a mapstate with some deer
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer1.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    deer1.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer1.components.evolving_traits = Some(EvolvingTraitsComponent {
        traits: EvolvingTraits{
            thick_hide: 50,
            sharp_claws: 50,
            ..Default::default()
        },
        ..Default::default()
    });

    deer1.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut deer2 =CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer2.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 2}
    };
    deer2.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 10,
        max_health: SIMPLE_ATTACK_BASE_DMG * 10,
    });
    deer2.components.evolving_traits = Some(EvolvingTraitsComponent {
        traits: EvolvingTraits{
            thick_hide: 10,
            sharp_claws: 150,
            ..Default::default()
        },
        ..Default::default()
    });

    println!("simple attack: {} starting HP: {} sharp ratio: {} hide: {}", SIMPLE_ATTACK_BASE_DMG, STANDARD_HP, SHARP_CLAWS_DMG_INCREASE, THICK_HIDE_DMG_REDUCE_MULTIPLIER);

    println!("deer1 attack: {} defense: {} sharp_claw {} hide: {}", 
        deer1.components.evolving_traits.as_ref().unwrap().get_total_simple_attack_adder(), 
        deer1.components.evolving_traits.as_ref().unwrap().get_total_defense_subtractor(), 
        deer1.components.evolving_traits.as_ref().unwrap().traits.sharp_claws, 
        deer1.components.evolving_traits.as_ref().unwrap().traits.thick_hide);
    println!("deer2 attack: {} defense: {} sharp_claw {} hide: {}", 
        deer2.components.evolving_traits.as_ref().unwrap().get_total_simple_attack_adder(), 
        deer2.components.evolving_traits.as_ref().unwrap().get_total_defense_subtractor(), 
        deer2.components.evolving_traits.as_ref().unwrap().traits.sharp_claws, 
        deer2.components.evolving_traits.as_ref().unwrap().traits.thick_hide);
    
    let deer1_id = deer1.components.id_component.id();
    let deer2_id = deer2.components.id_component.id();
    region.grid[deer1.components.location_component.location].creatures.add_creature(
        deer1, 0
    );
    region.grid[deer2.components.location_component.location].creatures.add_creature(
        deer2, 0
    );
    
    let attack = GoalNode {
        get_want_local: Box::new(|_, _| 10),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "attack",
        get_command: Some(Box::new(|m: & MapState, c| CreatureCommand::AttackSimple("attack_deer_closest", c, m.find_closest_creature_to_creature(c).unwrap()))),
        get_requirements_met: Box::new(|m, c| m.find_closest_creature_to_creature(c).is_some()),
    };
    //let root = GoalNode::generate_single_node_graph(attack);

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 2);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);
    assert_eq!(game_state.map_state.get_creature_item_list()[0].1, deer1_id);

    

    println!("creatures: {}", game_state.map_state.get_creature_strings());

    for _ in 0..7 {
        game_state = run_frame(game_state, &attack);
        println!("creatures: {}", game_state.map_state.get_creature_strings());
    }

    // deer1 should be dead and items on floor?
    assert_eq!(game_state.map_state.get_creature_list().len(), 1);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 1);
    
}

#[test]
fn test_soil_spread() {
    let openr = RegionCreationStruct::new(10,10, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut grass = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    grass.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    grass.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    grass.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    grass.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    // Just to make sure the grass doesn't replicate with the inventory
    grass.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    for row in &mut region.grid {
        for loc in row {
            loc.creatures.set_soil(SoilType::Clay);
        }
    }

    let grass_loc = grass.components.location_component.location;
    region.grid[grass_loc].creatures.set_soil(SoilType::Sand);

    region.grid[grass_loc].creatures.add_creature(
        grass, 0
    );
    let nothing = GoalNode::generate_single_node_graph();

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 1);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);
    let region_vu2 = Vu2::new(0,0);
    println!("{}", game_state.map_state.get_creature_map_strings(region_vu2));
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));
    for _ in 0..2 {
        game_state = run_frame(game_state, &nothing);
        //println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));

    // Is only 3 because bottom and left are blocked exits so you can't spread soil there even though technically they have creature list.
    assert_eq!(game_state.map_state.count_soils(region_vu2).sand_count, 3);

    // now add a budding component.
    let region: &mut MapRegion = &mut game_state.map_state.regions[0][0];
    region.grid[grass_loc].creatures.get_creature_by_index_mut(0).components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, 
        frame_ready_to_reproduce: 0, 
        seed_creature_differences: Box::new(ComponentMap::fake_default()), 
    });

    for _ in 0..26 {
        println!("Frame: {}", game_state.map_state.frame_count);
        game_state = run_frame(game_state, &nothing);
        //println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }
    println!("{}", game_state.map_state.get_creature_map_strings(region_vu2));
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));

    assert_eq!(game_state.map_state.count_soils(region_vu2).sand_count, 64);
    assert_eq!(game_state.map_state.get_creature_list().len(), 63); // last spot the soil is there but not yet budded as intended. Because of how event system works need a frame to spread soil then another to notice u can spread there.
}

#[test]
fn test_budding_height() {
    let openr = RegionCreationStruct::new(10,10, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut grass = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    grass.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    grass.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    grass.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    grass.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    grass.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    grass.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut flower = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    flower.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    flower.components.location_component = LocationComponent {
        location: Vu2{x: 7, y: 1}
    };
    flower.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    flower.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Flower,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    flower.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    flower.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut bush = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    bush.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    bush.components.location_component = LocationComponent {
        location: Vu2{x: 7, y: 1}
    };
    bush.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    bush.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Bush,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    bush.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    bush.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut tree = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree.components.location_component = LocationComponent {
        location: Vu2{x: 7, y: 1}
    };
    tree.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    tree.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 2, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    tree.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });


    let grass_loc = grass.components.location_component.location;
    region.grid[grass_loc].creatures.set_soil(SoilType::Sand);
    let flower_loc = grass.components.location_component.location;
    region.grid[flower_loc].creatures.set_soil(SoilType::Sand);
    let bush_loc = grass.components.location_component.location;
    region.grid[bush_loc].creatures.set_soil(SoilType::Sand);
    let tree_loc = grass.components.location_component.location;
    region.grid[tree_loc].creatures.set_soil(SoilType::Sand);

    region.grid[grass_loc].creatures.add_creature(
        grass, 0
    );
    region.grid[flower_loc].creatures.add_creature(
        flower, 0
    );
    region.grid[bush_loc].creatures.add_creature(
        bush, 0
    );
    region.grid[tree_loc].creatures.add_creature(
        tree, 0
    );


    let nothing = GoalNode::generate_single_node_graph();

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 4);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 4);
    let region_vu2 = Vu2::new(0,0);
    println!("{}", game_state.map_state.get_creature_map_strings(region_vu2));
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));
    for _ in 0..2 {
        game_state = run_frame(game_state, &nothing);
        //println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }

    println!("{}", game_state.map_state.get_creature_map_strings(region_vu2));
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));

    // Is only 3 because bottom and left are blocked exits so you can't spread soil there even though technically they have creature list.
    //assert_eq!(game_state.map_state.count_soils(region_vu2).sand_count, 3);

    return;

    for _ in 0..26 {
        println!("Frame: {}", game_state.map_state.frame_count);
        game_state = run_frame(game_state, &nothing);
        //println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }
    println!("{}", game_state.map_state.get_creature_map_strings(region_vu2));
    println!("{}", game_state.map_state.get_soil_map_strings(region_vu2));

    assert_eq!(game_state.map_state.count_soils(region_vu2).sand_count, 64);
    assert_eq!(game_state.map_state.get_creature_list().len(), 63); // last spot the soil is there but not yet budded as intended. Because of how event system works need a frame to spread soil then another to notice u can spread there.
}


// TODONEXT: Update the below budding functions to make sure they work with the new budding
// soil height and soil type stuff.
// Might want to make a separate tiny test for just spreading.
// make another test where everything is the right soil type. and just make sure there are 2 
// plants per square of diff height. and maybe check the ALL one only has 1.
// Then make a test where left half is one soil type, right half is other.
// and see if it only spread to the right soil? (3 plant types with no spreading?)
#[test]
fn test_chain_budding_system_one_of_each_soil<'a>() {
    let soil1 = SoilHeight::Bush;
    let soil2 = SoilHeight::Flower;
    let soil3 = SoilHeight::Bush;
    assert_eq!(soil1, soil3);
    assert_ne!(soil1, soil2);

    // make a mapstate with some budders
    let openr = RegionCreationStruct::new(10,10, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut grass = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    grass.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    grass.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    grass.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    grass.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    grass.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        ..Default::default()
    });
    // Just to make sure the grass doesn't replicate with the inventory
    grass.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut flower = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    flower.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    flower.components.location_component = LocationComponent {
        location: Vu2{x: 8, y: 1}
    };
    flower.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    flower.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    flower.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Flower,
        ..Default::default()
    });

    let mut bush = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    bush.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    bush.components.location_component = LocationComponent {
        location: Vu2{x: 5, y: 8}
    };
    bush.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    bush.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    bush.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Bush,
        ..Default::default()
    });
    
    region.grid[grass.components.location_component.location].creatures.add_creature(
        grass, 0
    );
    region.grid[flower.components.location_component.location].creatures.add_creature(
        flower, 0
    );
    region.grid[bush.components.location_component.location].creatures.add_creature(
        bush, 0
    );
    
    let nothing = GoalNode::generate_single_node_graph();

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 3);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);

    println!("{}", game_state.map_state.get_creature_map_strings(Vu2::new(0,0)));

    for _ in 0..3 {
        game_state = run_frame(game_state, &nothing);
        //println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }

    assert_eq!(game_state.map_state.get_creature_list().len(), 6);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);

    println!("{}", game_state.map_state.get_creature_map_strings(Vu2::new(0,0)));

    for _ in 0..60 {
        game_state = run_frame(game_state, &nothing);
    }
    
    println!("{}", game_state.map_state.get_creature_map_strings(Vu2::new(0,0)));

    // basically make sure all map points have exactly 3 creatures
    assert_eq!(game_state.map_state.get_creature_list().len(), 8*8*3);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 1);
    assert_eq!(game_state.map_state.regions[0][0].grid[4][5].creatures.get_length(), Some(3));
    assert_eq!(game_state.map_state.regions[0][0].grid[5][4].creatures.get_length(), Some(3));
    assert_eq!(game_state.map_state.regions[0][0].grid[1][1].creatures.get_length(), Some(3));
}

// Put some budding blockers. Also some deer. Watch the deer be moved around because of the trees
// Might be easiest to test by having a narrow region only 1 open wide
#[test]
fn test_chain_budding_system_blockers<'a>() {
    // make a mapstate with some budders
    let openr = RegionCreationStruct::new(10,3, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut tree = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree.components.block_space_component = Some(BlockSpaceComponent {});
    tree.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree.components.location_component = LocationComponent {
        location: Vu2{x: 3, y: 1}
    };
    tree.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    tree.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        ..Default::default()
    });    

    let mut tree2 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree2.components.block_space_component = Some(BlockSpaceComponent {});
    tree2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree2.components.location_component = LocationComponent {
        location: Vu2{x: 8, y: 1}
    };
    tree2.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree2.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    tree2.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        ..Default::default()
    });

    let mut tree3 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree3.components.block_space_component = Some(BlockSpaceComponent {});
    tree3.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree3.components.location_component = LocationComponent {
        location: Vu2{x: 4, y: 1}
    };
    tree3.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree3.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    tree3.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        ..Default::default()
    });

    let mut tree4 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree4.components.block_space_component = Some(BlockSpaceComponent {});
    tree4.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree4.components.location_component = LocationComponent {
        location: Vu2{x: 8, y: 1} // PURPOSELY put 2 in the same loc at the end to test blockers on same spot auto moving
    };
    tree4.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree4.components.budding_component = Some(BuddingComponent {
        reproduction_rate: 3,
        frame_ready_to_reproduce: 3,
        seed_creature_differences: Box::new(ComponentMap::fake_default()),
    });
    tree4.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        ..Default::default()
    });

    let mut deer1 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer1.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer1.components.location_component = LocationComponent {
        location: Vu2{x: 2, y: 1}
    };
    deer1.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    // See if it falls on death?
    deer1.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut deer2 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    deer2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    deer2.components.location_component = LocationComponent {
        location: Vu2{x: 5, y: 1}
    };
    deer2.components.health_component = Some(HealthComponent {
        health:  10,
        max_health: 10,
    });
    deer2.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });


    region.grid[tree.components.location_component.location].creatures.add_creature(
        tree, 0
    );
    region.grid[tree2.components.location_component.location].creatures.add_creature(
        tree2, 0
    );
    region.grid[tree3.components.location_component.location].creatures.add_creature(
        tree3, 0
    );
    region.grid[tree4.components.location_component.location].creatures.add_creature(
        tree4, 0
    );
    
    region.grid[deer1.components.location_component.location].creatures.add_creature(
        deer1, 0
    );
    region.grid[deer2.components.location_component.location].creatures.add_creature(
        deer2, 0
    );
    
    let nothing = GoalNode::generate_single_node_graph();

    let mut game_state = GameState {
        map_state:map
    };
    assert_eq!(game_state.map_state.get_creature_list().len(), 6);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 2);

    println!("replicated {}\n{}\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));

    for _ in 0..3 {
        game_state = run_frame(game_state, &nothing);
    }

    println!("replicated {}\n{}\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));

    //assert_eq!(game_state.map_state.get_creature_list().len(), 8);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 2);

    for _ in 0..3 {
        game_state = run_frame(game_state, &nothing);
    }
    
    println!("replicated {}\n{}\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));
    println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    //assert_eq!(game_state.map_state.get_creature_list().len(), 9);

    for _ in 0..3 {
        game_state = run_frame(game_state, &nothing);
    }
    
    println!("replicated {}\n{}\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));
    println!("\ncreatures:{}", game_state.map_state.get_creature_strings());

    assert_eq!(game_state.map_state.get_creature_list().len(), 8);
    assert_eq!(game_state.map_state.get_ground_item_list()[0].0.quantity, 2);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 0);
}


