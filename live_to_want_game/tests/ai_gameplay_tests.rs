extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::{*, reward_graph::{RootNode, Node, RewardNode, RewardResult, RequirementResult, VariableChange, CostResult, RewardNodeConnection, Variable, ConnectionResult, RewardNodeCreatureList}};
use strum::IntoEnumIterator;

#[test]
fn test_eat_soil_creatures() {
    // Create reward graph that has:
    // Move to Food (creature list) -> attack food (creature list) -> pick up food items -> eat items
    // eat items should be only thing with reward. rest use the connection stuff minus by their effort level
    let use_food = Node::Reward(RewardNode {
        description: "use_PSiltGrass".to_string(),
        index: 0,
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::InventoryItem(ItemType::PSiltGrass), 
            change: 1,
        }]],
        static_children: vec![], 
        reward: Box::new(|_, c, _| {
            let item_type = ItemType::PSiltGrass;
            RewardResult{
                reward_local: c.components.evolving_traits.as_ref().unwrap().get_calories_from_item_type(&item_type) as f32,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }),
        requirement: Box::new(|_, c| {
            RequirementResult {
                valid: c.get_inventory_of_item(ItemType::PSiltGrass) >= 1,
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
        get_command: Some(Box::new(|_, c,_,_| CreatureCommand::UseItem("use plant", InventoryHolder::CreatureInventory(c), Item::new(ItemType::PSiltGrass, 1)))), 
        effect: None,
        }
    );

    let pick_up_food = Node::Reward(RewardNode { 
        description: "pickup_PSiltGrass".to_string(),
        index: 1,
        static_requirements: vec![vec![VariableChange{ 
            variable: reward_graph::Variable::ProduceItem(ItemType::PSiltGrass), 
            change: 1,
        }]],
        static_children: vec![RewardNodeConnection{ 
            base_multiplier: None,
            child_index: 0, 
            parent_index: 1, 
            category: Variable::InventoryItem(ItemType::PSiltGrass),
            dont_match_targets: false,
        }],
        reward: Box::new(|_, _, _| {
            RewardResult{
                reward_local: 0., // use child reward
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _| {
            1.
        }),
        requirement: Box::new(|m, c| {
            let items = m.find_items_in_range_to_creature(c, 2.);
            for item in items.iter() {
                if item.item.item_type == ItemType::PSiltGrass {
                    return RequirementResult {
                        valid: true,
                        dynamic_and_static_requirements: vec![vec![]],
                        target_id: None,
                        target_location: Some(item.location),
                    };
                }
            }
            return RequirementResult {
                valid: false,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            };
        }), 
        cost: Box::new(|_, _, _| {
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }),
        get_command: Some(Box::new(|m, c, _, req_result| {
            if req_result.valid {
                let victim = m.location_to_map_location(&req_result.target_location.unwrap());
                let quantity = victim.get_inventory_of_item(ItemType::PSiltGrass);
                return CreatureCommand::TakeItem("take plant",InventoryHolder::LocationInventory(victim), InventoryHolder::CreatureInventory(c), Item::new(ItemType::PSiltGrass, quantity));
            }
            panic!("impossible getting command when req false");
        })),
        effect: Some(Box::new(|_m, _c, _reward, _requirement | {
            // Should be STATIC 1. Because the nodes connecting to this will do the multiplication based on amount
            // also if this does not produce any result sometimes, the child will panic because the category
            // wont match with the effect.
            return vec![VariableChange { 
                variable: Variable::InventoryItem(ItemType::PSiltGrass), 
                change: 1, 
            }]
        })),
        }
    );

    // Need to make it so connections are auto made based on what creature drops dynamically.
    // This can be done by making the effect something and the (would be) child node requirement match.
    // so effect: Produce(Grass) -> auto links with any node with requirement: Produce(Grass)
    let list_kill_node = Node::CreatureList(RewardNodeCreatureList {
        static_requirements: vec![vec![]],
        description: "list_kill_node".to_string(),
        index: 2, 
        static_children: vec![],
        reward: Box::new(|_, _, _, _other| {
            RewardResult{
                reward_local: 0.,
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _count, _| {
                1.
            // 9,9,9, 1
        }), 
        requirement: Box::new(|_, c, other| {
            RequirementResult {
                valid: c.get_if_in_melee_range(other.get_location()),
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }),
        cost: Box::new(|_, _, _, _| { // total reward should be 10 with these costs
            CostResult {
                cost_base: 1., // so u dont just kill also pick up
                cost_divider: 1.,
            }
        }), 
        get_command: Some(Box::new(|_map, c,_,_, other| {
            CreatureCommand::AttackSimple("attacking_test", c, other)
        }
        )),
        effect: Some(Box::new(|_map, _creature, _reward, _requirement, target | {
            target.get_variable_change_on_death()
        })),
        filter: Box::new(|_, c1, other| {
            if other.get_id() == c1.get_id() {
                return false;
            }
            return true;
        }),
        }
    );

    // using requirements to make connections, BUT ALSO they are used
    // for the fucking limit-algo. So I think it can conflict a lot. Need to make separate stuff?
    // or maybe its already a bit separate cause of the VariableChange in RewardConnection?
    // only when an item is added to inventory should it have requirements (pickup/craft)?
    // OKAY the issue is nodes that use an item to craft. So for example, if we can use
    // siltGrass to make a String item. then StringItem needs the SiltGrass requirement but
    // that would make it connect to Kill node instead of pickup node!
    // need to have two separate requirements? One for auto connections one for consuming requirements?
    // wait why not get rid of the pickup Node? oh because we need its action?
    // OR make a new Variable that is "Pickup"? so effect of kill is: Pickup(X)
    // then pickup effect is SiltGrass.
    // EffectEnum: Produce(Variable), Inventory(Variable)
    // Change all requirement stuff uses to this?
    // Wait can I just make Variable take in a fucking item as an enum? then can simplify conversions?
    // can wrap that in Produce/Inventory too.

    // move to creaturelist node. requirement is none. but reward is based on child of attack node.
    let move_to_node = Node::CreatureList(RewardNodeCreatureList {
        static_requirements: vec![vec![]],
        description: "move_to_node".to_string(),
        index: 3, 
        static_children: vec![RewardNodeConnection { 
            base_multiplier: Some(1.0), 
            child_index: 2, 
            parent_index: 3, 
            category: Variable::None,
            dont_match_targets: false,
        }],
        reward: Box::new(|_, _, _, _other| {
            RewardResult{
                reward_local: 0.,
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
                valid: true,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            }
        }),
        cost: Box::new(|_, c, _, other| { // total reward should be 10 with these costs
            CostResult {
                cost_base: c.get_location().distance_in_region(&other.get_location()).unwrap() as f32 / 10.0, // prioritise close by
                cost_divider: 1.,
            }
        }),
        get_command: Some(Box::new(|map, c,_,_, other| {
            CreatureCommand::MoveTo("move_to_creature", c, other.get_location(), map.frame_count)
        }
        )),
        effect: None,
        filter: Box::new(|_, c1, other| {
            if other.get_id() == c1.get_id() {
                return false;
            }
            return true;
        }),
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![use_food, pick_up_food, list_kill_node, move_to_node],
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

    let traits = EvolvingTraits {
        eat_sand_silt: 0,
        eat_sand_clay: 0,
        eat_silt_clay: 0,
        eat_grass_flower: 100,
        eat_grass_bush: 0,
        eat_grass_all: 0,
        eat_flower_bush: 0,
        eat_flower_all: 0,
        eat_bush_all: 0,
        far_sight: 200,
        ..Default::default()
    };
    // Test if they are all unique.
    let evolving_traits = EvolvingTraitsComponent {
        adult_traits: traits.clone(),
        traits: traits,
        child_until_frame: 0,
        born_on_frame: 0,
    };

    // Now make a region with a bunch of plants and a deer with:
    // a starving component, ai component, and evolving traits component, movement component
    // The deer should run around eating plants
    // maybe make it so the plants don't bud so its simpler to predict

    // run it for awhile, deer should run around and calories should change up and down when it eats

    let openr = RegionCreationStruct::new(10,10, 0, vec![]);
    let rgrid = vec![
        vec![openr],
    ];

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
        location: Vu2{x: 6, y: 1}
    };
    grass.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 3,
        max_health: SIMPLE_ATTACK_BASE_DMG * 3,
    });
    grass.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: None,
    });
    grass.inventory.push(Item::new(ItemType::PSiltGrass, 1));
    // Just to make sure the grass doesn't replicate with the inventory
    grass.components.death_items_component = Some(
        DeathItemsComponent { items_to_drop: vec![Item::new(ItemType::PSiltGrass, 1)] }
    );

    let mut grass2 = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    grass2.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    grass2.components.location_component = LocationComponent {
        location: Vu2{x: 8, y: 1}
    };
    grass2.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 3,
        max_health: SIMPLE_ATTACK_BASE_DMG * 3,
    });
    grass2.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: None,
    });
    grass2.inventory.push(Item::new(ItemType::PSiltGrass, 1));
    // Just to make sure the grass doesn't replicate with the inventory
    grass2.components.death_items_component = Some(
        DeathItemsComponent { items_to_drop: vec![Item::new(ItemType::PSiltGrass, 1)] }
    );

    let mut bush = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    bush.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    bush.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 4}
    };
    bush.components.health_component = Some(HealthComponent {
        health:  SIMPLE_ATTACK_BASE_DMG * 3,
        max_health: SIMPLE_ATTACK_BASE_DMG * 3,
    });
    bush.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Bush,
        soil_type_cannot_grow: SoilType::Silt,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: None,
    });
    bush.inventory.push(Item::new(ItemType::PSiltBush, 1));
    // Just to make sure the grass doesn't replicate with the inventory
    bush.components.death_items_component = Some(
        DeathItemsComponent { items_to_drop: vec![Item::new(ItemType::PSiltBush, 1)] }
    );

    for row in &mut region.grid {
        for loc in row {
            loc.creatures.set_soil(SoilType::Clay);
        }
    }


    //evolving_traits
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
    deer1.components.evolving_traits = Some(evolving_traits);
    deer1.components.movement_component = Some(MovementComponent {
        frames_to_move: STANDARD_FRAMES_TO_MOVE as usize,
        destination: Location::new(Vu2 { x: 0, y: 0 }, Vu2 { x: 0, y: 0 },),
        frame_ready_to_move: 0,
        moving: false,
    });
    deer1.components.starvation_component = Some(StarvationComponent { calories: 1000, metabolism: 20 });
    deer1.components.ai_component = Some(AIComponent { is_enabled_ai: true });
    deer1.components.vision_component = Some(VisionComponent { visible_creatures: vec![] });

    let creatures = vec![grass, grass2, bush, deer1];
    for creature in creatures {
        region.grid[creature.get_location().position].creatures.add_creature(
            creature, 0
        );
    }

    let mut game_state = GameState {
        map_state:map
    };
    
    println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    for _ in 0..80 {
        println!("Frame: {}", game_state.map_state.frame_count);
        println!("{}", game_state.map_state.get_creature_map_strings(Vu2 { x: 0, y: 0 }));
        game_state = run_frame(game_state, None, Some(&root));
        //println!("{:#?}", game_state.map_state.debug_info.as_ref().unwrap().ai[0]);

        // TODONEXT: Attack range seems too far wtf? can hit 2 tiles away.
        // also the above is awkward because what if an item is dropped too far away from you to pick up, need to be able to move to item->pickup. Maybe can just put it in the command itself for pickup. If too far to pickup->move to.
        // both grass killed by frame 53.
    }
    println!("{}", game_state.map_state.get_creature_map_strings(Vu2 { x: 0, y: 0 }));
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

    let soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });

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
    tree.components.soil_component = soil_component.clone();

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
    tree2.components.soil_component = soil_component.clone();

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
    tree3.components.soil_component = soil_component.clone();

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
    tree4.components.soil_component = soil_component.clone();

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

    println!("replicated {}\nAll:\n{}\nBlockers:\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));

    for _ in 0..3 {
        game_state = run_frame(game_state, Some(&nothing), None);
    }

    println!("replicated {}\nAll:\n{}\nBlockers:\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));

    //assert_eq!(game_state.map_state.get_creature_list().len(), 8);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 2);

    for _ in 0..3 {
        game_state = run_frame(game_state, Some(&nothing), None);
    }
    
    println!("replicated {}\nAll:\n{}\nBlockers:\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));
    println!("\ncreatures all:{}", game_state.map_state.get_creature_strings());
    //assert_eq!(game_state.map_state.get_creature_list().len(), 9);

    for _ in 0..3 {
        game_state = run_frame(game_state, Some(&nothing), None);
    }
    
    println!("replicated {}\nAll:\n{}\nBlockers:\n{}", game_state.map_state.frame_count/3, game_state.map_state.get_creature_map_strings(Vu2::new(0,0)), 
        game_state.map_state.get_creature_map_strings_filtered(Vu2::new(0,0), &|c: &&CreatureState| c.components.block_space_component.is_some()));
    println!("\ncreatures all:{}", game_state.map_state.get_creature_strings());

    assert_eq!(game_state.map_state.get_creature_list().len(), 8);
    assert_eq!(game_state.map_state.get_ground_item_list()[0].0.quantity, 2);
    assert_eq!(game_state.map_state.get_creature_item_list().len(), 0);
}


