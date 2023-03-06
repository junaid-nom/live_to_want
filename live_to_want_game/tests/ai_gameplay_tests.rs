extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::{*, reward_graph::{RootNode, Node, RewardNode, RewardResult, RequirementResult, VariableChange, CostResult, RewardNodeConnection, Variable, ConnectionResult, RewardNodeCreatureList}};
use strum::IntoEnumIterator;

#[test]
fn test_eat_soil_creatures() {
    // Create reward graph that has:
    // Find Food (wander) -> Chase Food (creature list) -> attack food -> pick up food items -> eat items
    // eat items should be only thing with reward? rest use the connection stuff divide by their effort level
    let use_food = Node::Reward(RewardNode { 
        description: "use_food".to_string(),
        index: 0, 
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
                dynamic_and_static_requirements: vec![vec![VariableChange{ 
                    variable: reward_graph::Variable::PSiltGrass, 
                    change: 1
                }]],
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
        description: "use_food".to_string(),
        index: 1,
        static_children: vec![RewardNodeConnection{ base_multiplier: Some(1.), child_index: 0, parent_index: 1, requirement: VariableChange { variable: Variable::PSiltGrass, change: 1 } }], 
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
        effect: None,
        }
    );

    // attack nearby plant requirement is nearby plant creature. cost is lets say divisor of 2
    // local reward is 0. command is attack target mentioned in requirement result.
    // or maybe use creaturelist? and just requirement is they are in range of 2?
    // fuck idk in real game prob need 1 kill node that attaches to every item drop possible?
    // I think I have to use the effect and requirement variable change stuff??
    // TODO: make base_multiplier in children, a function. then it can do stuff like
    // multiplier = drop + inventory
    // TODO: ACtually I think all I have to do is set the reward multiplier to None
    // then set the effect to be everything the creature will drop.
    // and set the requirement of "pick up item" to be 1 of the item.
    // and then it'll work? Should make a separate test to confirm? Or maybe this test
    // will confirm for me

    // move to creaturelist node. requirement is none? but reward is based on child of attack node. 

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![use_food, pick_up_food],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 3, 
                parent_index: 0,
                requirement: VariableChange { variable: Variable::None, change: 0 }
            },
        ],
    };

    let traits = EvolvingTraits {
        eat_sand_silt: 1,
        eat_sand_clay: 0,
        eat_silt_clay: 1,
        eat_grass_flower: 1,
        eat_grass_bush: 1,
        eat_grass_all: 0,
        eat_flower_bush: 0,
        eat_flower_all: 0,
        eat_bush_all: 1,
        ..Default::default()
    };
    // Test if they are all unique.
    let component = EvolvingTraitsComponent {
        adult_traits: traits.clone(),
        traits: traits,
        child_until_frame: 0,
        born_on_frame: 0,
    };
    assert_eq!(4, component.get_calories_from_item_type(&ItemType::PSiltGrass));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSiltFlower));
    assert_eq!(4, component.get_calories_from_item_type(&ItemType::PSiltBush));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSiltAll));
    
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSandGrass));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSandFlower));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSandBush));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSandAll));
    
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PClayGrass));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PClayFlower));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PClayBush));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PClayAll));

    // inverse of above traits
    let traits = EvolvingTraits {
        eat_sand_silt: 0,
        eat_sand_clay: 1,
        eat_silt_clay: 0,
        eat_grass_flower: 0,
        eat_grass_bush: 0,
        eat_grass_all: 1,
        eat_flower_bush: 1,
        eat_flower_all: 1,
        eat_bush_all: 0,
        ..Default::default()
    };
    // Test if they are all unique.
    let component = EvolvingTraitsComponent {
        adult_traits: traits.clone(),
        traits: traits,
        child_until_frame: 0,
        born_on_frame: 0,
    };
    assert_eq!(1, component.get_calories_from_item_type(&ItemType::PSiltGrass));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSiltFlower));
    assert_eq!(1, component.get_calories_from_item_type(&ItemType::PSiltBush));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSiltAll));
    
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSandGrass));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSandFlower));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PSandBush));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PSandAll));
    
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PClayGrass));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PClayFlower));
    assert_eq!(2, component.get_calories_from_item_type(&ItemType::PClayBush));
    assert_eq!(3, component.get_calories_from_item_type(&ItemType::PClayAll));
    

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


