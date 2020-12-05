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
