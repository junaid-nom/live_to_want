extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_movement() {
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
    let move_chain: Vec<EventChain> = map.regions[0][0].grid[1][1].creatures.get_par_iter().unwrap().map(|c| {
        CreatureCommand::MoveTo("test_move", c, Location::new(Vu2::new(2,1), Vu2::new(3,3)), 0).to_event_chain().unwrap()
    }).collect();
    
    //process init EC
    process_events_from_mapstate(&mut map, move_chain);

    //in loop:
    for frame_add in 0..2 {
        map.frame_count+=1;
        println!("Starting frame: {}", map.frame_count);
        //run movement system
        let mov_op_ecs: Vec<Option<EventChain>> = map.regions.par_iter().flat_map(|x| {
            x.par_iter().flat_map(|y| {
                y.grid.par_iter().flat_map(|xl| {
                    xl.par_iter().flat_map(|yl| {
                        if let Some(cit) = yl.creatures.get_par_iter() {
                            let ret: Vec<Option<EventChain>> = cit.map(
                                |c| {
                                    movement_system(&map, c)
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
        process_events_from_mapstate(&mut map, event_chains);
    }
    // make sure creature location changes to what it shud be

}

