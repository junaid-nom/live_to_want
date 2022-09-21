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

// TODO: Make test for metabolism that checks to see if traits and if moving stuff works.
// Prob can just postpone for awhile and do 1 test that uses EVERY trait that changes them and make 1 big calculation.

// TODONEXT: Test sex, and then reproduction. Make sure the sex related stuff like species, multithreads, mutating, inheritance, litter size, pregnancy time, and childness work.
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
fn test_chain_budding_system_one_of_each_soil<'a>() {
    let soil1 = SoilLayer::Bush;
    let soil2 = SoilLayer::Flower;
    let soil3 = SoilLayer::Bush;
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
        soil_layer: SoilLayer::All
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
        soil_layer: SoilLayer::All
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
        soil_layer: SoilLayer::All
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
        soil_layer: SoilLayer::All
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


