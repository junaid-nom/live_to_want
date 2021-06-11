extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

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
        items_to_drop: vec![Item::new(ItemType::Bones, 7)],
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
            item_type: ItemType::Bones,
            quantity: 7,
        },
        Item {
            item_type: ItemType::Meat,
            quantity: 6,
        },
    ], gs.map_state.regions[start_loc].items);
}

// TODONEXT: The budding system is likely broken/untested! Especially The part involving blockers and non blocker stuff! 
#[test]
fn test_chain_budding_system_one_of_each_soil<'a>() {
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
        soil_layer: SoilLayer::Grass
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
        soil_layer: SoilLayer::Flower
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
        soil_layer: SoilLayer::Bush
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

    for _ in 0..1 {
        game_state = run_frame(game_state, &nothing);
        println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }

    assert_eq!(game_state.map_state.get_creature_list().len(), 3);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);

    // println!("BEGIN PHASE 2!");

    for _ in 0..1 {
        game_state = run_frame(game_state, &nothing);
        println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }

    assert_eq!(game_state.map_state.get_creature_list().len(), 3);
    assert_eq!(game_state.map_state.get_creature_list()[0].components.health_component.as_ref().unwrap().health, 1);
    assert_eq!(game_state.map_state.get_ground_item_list().len(), 0);

    for _ in 0..1 {
        game_state = run_frame(game_state, &nothing);
        println!("\ncreatures:{}", game_state.map_state.get_creature_strings());
    }
}
