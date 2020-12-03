extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_movement_normal() {
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    
    let rgrid = vec![
        vec![openr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    //make creature
    let mut c = CreatureState::new_location(Location::new(Vu2::new(0,0), Vu2::new(1,1)));
    c.components.movement_component = Some(MovementComponent{
        frames_to_move:2,
        moving:false,
        frame_ready_to_move:0,
        destination:Location::new0(),
    });
    println!("Creature id: {}", c.components.id_component.id());

    map.regions[0][0].grid[1][1].creatures.add_creature(c, 0);
    println!("0,0-1,1 id: {}", map.regions[0][0].grid[1][1].id_component_creatures.id());
    //create command to move
    //generate event chain init movement
    let dst_region = Vu2::new(2,1);
    let dst_position = Vu2::new(3,3);
    let move_chain: Vec<EventChain> = map.regions[0][0].grid[1][1].creatures.get_par_iter().unwrap().map(|c| {
        CreatureCommand::MoveTo("test_move", c, Location::new(dst_region, dst_position), 0).to_event_chain().unwrap()
    }).collect();
    
    //process init EC
    process_events_from_mapstate(&mut map, move_chain, false);

    //in loop:
    for frame_add in 0..38 {
        map.frame_count+=1;
        let current_frame = map.frame_count;
        println!("Starting frame: {}", map.frame_count);
        //run movement system
        let mov_op_ecs: Vec<Option<EventChain>> = map.regions.par_iter().flat_map(|x| {
            x.par_iter().flat_map(|y| {
                y.grid.par_iter().flat_map(|xl| {
                    xl.par_iter().flat_map(|yl| {
                        if let Some(cit) = yl.creatures.get_par_iter() {
                            let ret: Vec<Option<EventChain>> = cit.map(
                                |c| {
                                    movement_system_move(&map, c)
                                }
                            ).collect();
                            return ret;
                        } else {
                            Vec::new()
                        }
                    })
                })
            })
        }).collect();
        //process movement event if there are any
        let mut event_chains: Vec<EventChain> = unwrap_option_list(mov_op_ecs);
        process_events_from_mapstate(&mut map, event_chains, true);

        // iterate on movement system for each creature
        map.regions.par_iter_mut().for_each(|x| {
            x.par_iter_mut().for_each(|y| {
                let region_loc = y.location;
                y.grid.par_iter_mut().for_each(|xl| {
                    xl.par_iter_mut().for_each(|yl| {
                        let position = yl.location;
                        let location = Location::new(region_loc, position);
                        if let Some(cit) = yl.creatures.get_par_iter_mut() {
                            cit.for_each(
                                |c| {
                                    movement_system_iterate(current_frame, c, location);
                                }
                            );
                        }
                    })
                })
            })
        });
    }
    // TODONEXT: make sure creature location changes to what it shud be
    // also make sure movement is stopped
    println!("creatures at target: {:#?}", map.regions[dst_region].grid[dst_position].creatures);
    let creature = map.regions[dst_region].grid[dst_position].creatures.get_creature_by_index(0);
    assert_eq!(creature.components.movement_component.as_ref().unwrap().moving, false);
    assert_eq!(creature.components.region_component.region, Vu2::new(2,1));
    assert_eq!(creature.components.location_component.location, Vu2::new(3,3));
}

#[test]
fn test_movement_closer_region() {
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    
    let rgrid = vec![
        vec![openr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    //make creature
    let start_loc = Location::new(Vu2::new(0,0), Vu2::new(2,1));
    let mut c = CreatureState::new_location(start_loc);
    c.components.movement_component = Some(MovementComponent{
        frames_to_move:2,
        moving:false,
        frame_ready_to_move:0,
        destination:Location::new0(),
    });
    println!("Creature id: {}", c.components.id_component.id());

    map.regions[start_loc.region].grid[start_loc.position].creatures.add_creature(c, 0);
    println!("{:?} id: {}", start_loc, map.regions[start_loc.region].grid[start_loc.position].id_component_creatures.id());
    //create command to move
    //generate event chain init movement
    let dst_region = Vu2::new(2,1);
    let dst_position = Vu2::new(3,3);
    let move_chain: Vec<EventChain> = map.regions[start_loc.region].grid[start_loc.position].creatures.get_par_iter().unwrap().map(|c| {
        CreatureCommand::MoveTo("test_move", c, Location::new(dst_region, dst_position), 0).to_event_chain().unwrap()
    }).collect();
    
    //process init EC
    process_events_from_mapstate(&mut map, move_chain, false);

    //in loop:
    for frame_add in 0..8 {
        map.frame_count+=1;
        let current_frame = map.frame_count;
        println!("Starting frame: {}", map.frame_count);
        //run movement system
        let mov_op_ecs: Vec<Option<EventChain>> = map.regions.par_iter().flat_map(|x| {
            x.par_iter().flat_map(|y| {
                y.grid.par_iter().flat_map(|xl| {
                    xl.par_iter().flat_map(|yl| {
                        if let Some(cit) = yl.creatures.get_par_iter() {
                            let ret: Vec<Option<EventChain>> = cit.map(
                                |c| {
                                    movement_system_move(&map, c)
                                }
                            ).collect();
                            return ret;
                        } else {
                            Vec::new()
                        }
                    })
                })
            })
        }).collect();
        //process movement event if there are any
        let mut event_chains: Vec<EventChain> = unwrap_option_list(mov_op_ecs);
        process_events_from_mapstate(&mut map, event_chains, true);

        // iterate on movement system for each creature
        map.regions.par_iter_mut().for_each(|x| {
            x.par_iter_mut().for_each(|y| {
                let region_loc = y.location;
                y.grid.par_iter_mut().for_each(|xl| {
                    xl.par_iter_mut().for_each(|yl| {
                        let position = yl.location;
                        let location = Location::new(region_loc, position);
                        if let Some(cit) = yl.creatures.get_par_iter_mut() {
                            cit.for_each(
                                |c| {
                                    movement_system_iterate(current_frame, c, location);
                                }
                            );
                        }
                    })
                })
            })
        });
    }
    // TODONEXT: make sure creature location changes to what it shud be
    // also make sure movement is stopped
    println!("creatures at target: {:#?}", map.regions[1][0].grid[0][2].creatures);
    let creature = map.regions[1][0].grid[0][2].creatures.get_creature_by_index(0);
    assert_eq!(creature.components.movement_component.as_ref().unwrap().moving, true);
    assert_eq!(creature.components.region_component.region, Vu2::new(1,0));
    assert_eq!(creature.components.location_component.location, Vu2::new(0,2));
}

#[test]
fn test_movement_fat_region() {
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    let fatr = RegionCreationStruct::new(15,15, 0, vec![]);

    let rgrid = vec![
        vec![openr.clone(),openr.clone()],
        vec![fatr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    //make creature
    let mut c = CreatureState::new_location(Location::new(Vu2::new(0,0), Vu2::new(1,1)));
    c.components.movement_component = Some(MovementComponent{
        frames_to_move:2,
        moving:false,
        frame_ready_to_move:0,
        destination:Location::new0(),
    });
    println!("Creature id: {}", c.components.id_component.id());

    map.regions[0][0].grid[1][1].creatures.add_creature(c, 0);
    println!("0,0-1,1 id: {}", map.regions[0][0].grid[1][1].id_component_creatures.id());
    //create command to move
    //generate event chain init movement
    let dst_region = Vu2::new(2,0);
    let dst_position = Vu2::new(3,1);
    let move_chain: Vec<EventChain> = map.regions[0][0].grid[1][1].creatures.get_par_iter().unwrap().map(|c| {
        CreatureCommand::MoveTo("test_move", c, Location::new(dst_region, dst_position), 0).to_event_chain().unwrap()
    }).collect();
    
    //process init EC
    process_events_from_mapstate(&mut map, move_chain, false);

    //in loop:
    for frame_add in 0..40 {
        map.frame_count+=1;
        let current_frame = map.frame_count;
        println!("Starting frame: {}", map.frame_count);
        //run movement system
        let mov_op_ecs: Vec<Option<EventChain>> = map.regions.par_iter().flat_map(|x| {
            x.par_iter().flat_map(|y| {
                y.grid.par_iter().flat_map(|xl| {
                    xl.par_iter().flat_map(|yl| {
                        if let Some(cit) = yl.creatures.get_par_iter() {
                            let ret: Vec<Option<EventChain>> = cit.map(
                                |c| {
                                    movement_system_move(&map, c)
                                }
                            ).collect();
                            return ret;
                        } else {
                            Vec::new()
                        }
                    })
                })
            })
        }).collect();
        //process movement event if there are any
        let mut event_chains: Vec<EventChain> = unwrap_option_list(mov_op_ecs);
        process_events_from_mapstate(&mut map, event_chains, true);

        // iterate on movement system for each creature
        map.regions.par_iter_mut().for_each(|x| {
            x.par_iter_mut().for_each(|y| {
                let region_loc = y.location;
                y.grid.par_iter_mut().for_each(|xl| {
                    xl.par_iter_mut().for_each(|yl| {
                        let position = yl.location;
                        let location = Location::new(region_loc, position);
                        if let Some(cit) = yl.creatures.get_par_iter_mut() {
                            cit.for_each(
                                |c| {
                                    movement_system_iterate(current_frame, c, location);
                                }
                            );
                        }
                    })
                })
            })
        });
    }
    // TODONEXT: make sure creature location changes to what it shud be
    // also make sure movement is stopped
    println!("creatures at target: {:#?}", map.regions[dst_region].grid[dst_position].creatures);
    let creature = map.regions[dst_region].grid[dst_position].creatures.get_creature_by_index(0);
    assert_eq!(creature.components.movement_component.as_ref().unwrap().moving, false);
    assert_eq!(creature.components.region_component.region, dst_region);
    assert_eq!(creature.components.location_component.location, dst_position);
}
