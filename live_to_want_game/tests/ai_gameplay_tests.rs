extern crate rayon;
use std::{rc::Rc, cell::RefCell, collections::HashSet};

use rayon::prelude::*;
use live_to_want_game::{*, reward_graph::{RootNode, Node, RewardNode, RewardResult, RequirementResult, EffectChange, CostResult, RewardNodeConnection, Effect, ConnectionResult, RewardNodeList, NodeTargetType, NodeTarget}};
use strum::IntoEnumIterator;

#[test]
fn test_eat_soil_creatures() {
    // Create reward graph that has:
    // Move to Food (creature list) -> attack food (creature list) -> pick up food items -> eat items. Need to also be able to pick up items on the ground
    // eat items should be only thing with reward. rest use the connection stuff minus by their costs
    let use_food = Node::Reward(RewardNode {
        description: "use_eat_PSiltGrass".to_string(),
        index: 0,
        static_requirements: vec![vec![EffectChange{
            effect: reward_graph::Effect::HaveItem(ItemType::PSiltGrass), 
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


    // This would work with any item, but this example only has PSiltGrass items
    let pick_up_from_ground = Node::ListNode(RewardNodeList { 
        description: "pickup".to_string(),
        index: 1,
        static_requirements: vec![vec![]],
        target_types: HashSet::from([NodeTargetType::LocationItemTarget]),
        filter: Box::new(|_, _c1, _other| {
            return true;
        }),
        static_children: vec![],
        reward: Box::new(|_, _, _, _item_loc| {
            RewardResult{
                reward_local: 0., // use child reward
                target_id: None,
                target_location: None,
            }
        }),
        reward_connection: Box::new(|_, _, _, _| {
            1.
        }),
        requirement: Box::new(|_m, c, target| {
            let valid = 
                if let NodeTarget::LocationItemTarget(loc, _) = target {
                    c.get_if_in_melee_range(loc)
                } else {
                    false
                };
            return RequirementResult {
                valid,
                dynamic_and_static_requirements: vec![vec![]],
                target_id: None,
                target_location: None,
            };
        }),
        cost: Box::new(|_, _, _, _| {
            CostResult {
                cost_base: 0.,
                cost_divider: 1.,
            }
        }),
        get_command: Some(Box::new(|m, c, _, _req_result, item_loc| {
            if let NodeTarget::LocationItemTarget(loc, item_type) = item_loc {
                let victim = m.location_to_map_location(&loc);
                let quantity = victim.get_inventory_of_item(ItemType::PSiltGrass);
                return CreatureCommand::TakeItem("take plant",InventoryHolder::LocationInventory(victim), InventoryHolder::CreatureInventory(c), Item::new(item_type, quantity))
            } else {
                panic!("target not correct for pickup item ground");
            };
        })),
        effect: Some(Box::new(|m, _c, _reward, _requirement, item_loc| {
            let (loc, itype) = item_loc.as_location_item();
            let victim = m.location_to_map_location(&loc);
            let quantity = victim.get_inventory_of_item(ItemType::PSiltGrass);
            return vec![EffectChange::new(Effect::HaveItem(*itype), quantity as i32)]
        })),
        }
    );

    // Need to make it so connections are auto made based on what creature drops dynamically.
    // This can be done by making the effect something and the (would be) child node requirement match.
    // directly links to Use item even though when you kill it drops an item, because 
    // makes graph simpler than having an inbetween hypothetical pickup
    // so effect: Have(Grass) -> auto links with any node with requirement: Have(Grass)
    let list_kill_node = Node::ListNode(RewardNodeList {
        static_requirements: vec![vec![]],
        target_types: HashSet::from([NodeTargetType::CreatureTarget]),
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
                valid: c.get_if_in_melee_range(&other.as_creature().get_location()),
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
            CreatureCommand::AttackSimple("attacking_test", c, other.as_creature())
        }
        )),
        effect: Some(Box::new(|_map, _creature, _reward, _requirement, target | {
            target.as_creature().get_variable_change_on_death(false)
        })),
        filter: Box::new(|_, c1, other| {
            if other.as_creature().get_id() == c1.get_id() {
                return false;
            }
            return true;
        }),
        }
    );

    // move to creaturelist node. requirement is none. but reward is based on child connections
    let move_to_node = Node::ListNode(RewardNodeList {
        static_requirements: vec![vec![]],
        target_types: HashSet::from([NodeTargetType::CreatureTarget, NodeTargetType::LocationItemTarget]),
        description: "move_to_node".to_string(),
        index: 3, 
        static_children: vec![RewardNodeConnection { 
                base_multiplier: Some(1.0), 
                child_index: 2, 
                parent_index: 3, 
                category: Effect::None,
                dont_match_targets: false,
            },
            RewardNodeConnection { 
                base_multiplier: Some(1.0), 
                child_index: 1, 
                parent_index: 3, 
                category: Effect::None,
                dont_match_targets: false,
            }
        ],
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
        cost: Box::new(|_, c, _, other| {
            let loc = match other {
                NodeTarget::CreatureTarget(other) => other.get_location(),
                NodeTarget::LocationItemTarget(loc, _) => *loc,
                _ => panic!("invalid target for move node"),
            };
            CostResult {
                cost_base: c.get_location().distance_in_region(&loc).unwrap() as f32 / 10.0, // prioritise close by
                cost_divider: 1.,
            }
        }),
        get_command: Some(Box::new(|map, c,_,_, other| {
            let loc = match other {
                NodeTarget::CreatureTarget(other) => other.get_location(),
                NodeTarget::LocationItemTarget(loc, _) => loc,
                _ => panic!("invalid target for move node"),
            };
            CreatureCommand::MoveTo("move_to_creature", c, loc, map.frame_count)
        }
        )),
        effect: Some(Box::new(|_map, _creature, _reward, _requirement, target | {
            match target {
                reward_graph::NodeTarget::CreatureTarget(_) => vec![],
                reward_graph::NodeTarget::LocationItemTarget(_,_) => vec![],
                _ => panic!("invalid target for move node"),
            }
        })),
        filter: Box::new(|_, c1, other| {
            if let NodeTarget::CreatureTarget(other)  = other {
                if other.get_id() == c1.get_id() {
                    return false;
                }
            }
            return true;
        }),
        }
    );

    let root = RootNode{
        description: "root".to_string(),
        nodes: vec![use_food, pick_up_from_ground, list_kill_node, move_to_node],
        children: vec![
            RewardNodeConnection{ 
                base_multiplier: Some(1.), 
                child_index: 3, 
                parent_index: 0,
                category: Effect::None,
                dont_match_targets: false,
            },
            // Need to add a root node to the use item node as well so that you don't
            // require a item on ground or creature to eat the item
            RewardNodeConnection{
                base_multiplier: Some(1.), 
                child_index: 0, 
                parent_index: 0,
                category: Effect::None,
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
    // a starving component, ai component, and evolving traits component, movement componentm vision... etc
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

    let starting_calories = 1000;
    let metabolism = 10;
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
    deer1.components.starvation_component = Some(StarvationComponent { calories: starting_calories, metabolism: metabolism });
    deer1.components.ai_component = Some(AIComponent { is_enabled_ai: true });
    deer1.components.vision_component = Some(VisionComponent { visible_creatures: vec![] });

    let deer_id = deer1.get_id();
    let deer_grass_calories = deer1.components.evolving_traits.as_ref().unwrap().get_calories_from_item_type(&ItemType::PSiltGrass);

    let creatures = vec![grass, grass2, bush, deer1];
    for creature in creatures {
        region.grid[creature.get_location().position].creatures.add_creature(
            creature, 0
        );
    }
    region.grid[8][6].items.push(Item::new(ItemType::PSiltGrass, 1));
    let mut game_state = GameState {
        map_state:map
    };
    
    println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    let frames = 103;
    for _ in 0..frames {
        println!("Frame: {}", game_state.map_state.frame_count);
        println!("{}", game_state.map_state.get_creature_map_strings(Vu2 { x: 0, y: 0 }));
        game_state = run_frame(game_state, None, Some(&root));
        println!("{:#?}", game_state.map_state.debug_info.as_ref().unwrap().ai[0].final_node_descriptor);
        println!("Ground: {:#?}", game_state.map_state.get_ground_item_list());
        println!("Creature: {:#?}", game_state.map_state.get_creature_item_list());

        let creatures_map = game_state.map_state.get_creatures_hashmap();
        println!("Calories: {:#?} adult percent: {}", &creatures_map.get(&deer_id).unwrap().components.starvation_component.as_ref().unwrap().calories,  &creatures_map.get(&deer_id).unwrap().get_adult_percent(game_state.map_state.frame_count));
        // both grass killed and eaten by frame ~69? rest is to move to the ground item.
    }
    println!("{}", game_state.map_state.get_creature_map_strings(Vu2 { x: 0, y: 0 }));
    println!("{:#?}", game_state.map_state.debug_info.as_ref().unwrap().ai[0]);

    let creatures_map = game_state.map_state.get_creatures_hashmap();
    let calories: i32 = creatures_map.get(&deer_id).unwrap().components.starvation_component.as_ref().unwrap().calories;
    let num_grass_dropped = 5.; // 2 per grass creature 1 on ground
    let expected_calories = starting_calories as f32 - (frames as  f32 * metabolism as f32 * MOVING_INCREASED_METABOLISM_FACTOR) + (num_grass_dropped *deer_grass_calories as f32);
    println!("Calories: {:#?} expected: {}", calories, expected_calories);
    assert!(expected_calories < calories as f32);

    // NOTE we only have 1 useItem node for our specific grass item. 
    // in the future we may want to make a "ItemType" List node that will
    // foreach on all item types and essentially be UseItem on all item types.
    // Issue though is do we want all that code in 1 node? Might be better to keep
    // them split, especially if many items cannot even be used anyway.
}




