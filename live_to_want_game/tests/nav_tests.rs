extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_map_state_update() {
    // TODO Call update on map state changing just a little but of stuff each time to make sure it changes
    let xlen = 9;
    let ylen = 9;
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    let openbigr = RegionCreationStruct::new(xlen,ylen, 0, vec![]);


    let maze_creation_open = vec![
        Vu2::new(4, 0),
        Vu2::new(2, 1),Vu2::new(3, 1),Vu2::new(4, 1),
        Vu2::new(2, 2), Vu2::new(5, 2),Vu2::new(6, 2),Vu2::new(7, 2),
        Vu2::new(2, 3),Vu2::new(3, 3),Vu2::new(4, 3),Vu2::new(5, 3),Vu2::new(7, 3),
        Vu2::new(0, 4),Vu2::new(1, 4),Vu2::new(3, 4),Vu2::new(5, 4),Vu2::new(7, 4),Vu2::new(8, 4),
        Vu2::new(1, 5),Vu2::new(3, 5),Vu2::new(4, 5),Vu2::new(5, 5),Vu2::new(6, 5),
        Vu2::new(1, 6),Vu2::new(2, 6),Vu2::new(3, 6),Vu2::new(6, 6),
        Vu2::new(4, 6),Vu2::new(5, 6),Vu2::new(6, 6),
        Vu2::new(4, 8),
    ];

    let closer = RegionCreationStruct::new(0,0, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone(),openbigr.clone(),closer.clone(),openr.clone(),openr.clone()],
        vec![openbigr.clone(),openr.clone(),openr.clone(),closer.clone(),openr.clone()],
        vec![closer.clone(),openr.clone(),closer.clone(),closer.clone(),openr.clone()],
        vec![openr.clone(),openr.clone(),openr.clone(),openr.clone(),openr.clone()],
    ];

    let print_rc = || {
        for y in (0..rgrid[0].len()).rev() {
            for x in 0..rgrid.len() {
                if rgrid[x][y].xlen == 0 {
                    print!("X ");

                } else {
                    print!("O ");
                }
            }
            println!("");
        }
    };
    print_rc();
    let mut map = MapState::new(rgrid.clone(), 0);
    print_rc();
    println!("printing map:");
    println!("{}", map);
    let dst = Vu2::new(0, 3);
    let src = Vu2::new(0,1);
    println!("printing dist from {:#?}", dst);
    println!("{}", map.get_distance_strings(&dst));
    println!("{:#?}", map.regions[src].region_distances[dst]);
    // NOTE: It's okay if region_distances are based on a single direction.
    // in actual navigation algo can look at the distnaces to end of the neighbors and pick 
    // one randomly if they are the same instead!
    assert_eq!(map.regions[src].region_distances[dst], RegionSetDistances::Set(RegionDistances{
        left: None,
        right: Some(36),
        up: None,
        down: Some(44),
    }));

    for x in 0..xlen {
        for y in 0..ylen {
            let loc = Vu2::new(x,y);
            if !maze_creation_open.contains(&loc) {
                if map.regions[0][1].grid[loc].creatures.holds_creatures() {
                    // Note some edges are blocked because no neighbor so a few points in the path will be blocked
                    let mut new_creature = CreatureState::new(loc);
                    new_creature.components.block_space_component = Some(BlockSpaceComponent{});
                    map.regions[0][1].grid[loc].creatures.add_creature(new_creature, 1);
                }
            }
        }
    }
    map.regions[0][1].update_region_nav(1);
    map.update_nav();
    println!("printing dist after update from {:#?}", dst);
    println!("{}", map.get_distance_strings(&dst));
    println!("{:#?}", map.regions[src].region_distances[dst]);
    println!("{}", map.regions[src].to_string());
    assert_eq!(map.regions[src].region_distances[dst], RegionSetDistances::Set(RegionDistances{
        left: None,
        right: Some(36),
        up: None,
        down: Some(50),
    }));
    // make sure calling update more than once is deterministic and stuff
    map.update_nav();
    assert_eq!(map.regions[src].region_distances[dst], RegionSetDistances::Set(RegionDistances{
        left: None,
        right: Some(36),
        up: None,
        down: Some(50),
    }));
}

#[test]
fn test_map_state() {
    let openr = RegionCreationStruct::new(5,5, 0, vec![]);
    let closer = RegionCreationStruct::new(0,0, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone(),openr.clone(),closer.clone(),openr.clone(),openr.clone()],
        vec![openr.clone(),openr.clone(),openr.clone(),closer.clone(),openr.clone()],
        vec![closer.clone(),openr.clone(),closer.clone(),closer.clone(),openr.clone()],
        vec![openr.clone(),openr.clone(),openr.clone(),openr.clone(),openr.clone()],
    ];

    let print_rc = || {
        for y in (0..rgrid[0].len()).rev() {
            for x in 0..rgrid.len() {
                if rgrid[x][y].xlen == 0 {
                    print!("X ");

                } else {
                    print!("O ");
                }
            }
            println!("");
        }
    };
    print_rc();
    let map = MapState::new(rgrid.clone(), 0);
    print_rc();
    println!("printing map:");
    println!("{}", map);
    let dst = Vu2::new(0, 3);
    println!("printing dist from {:#?}", dst);
    println!("{}", map.get_distance_strings(&dst));
    println!("{:#?}", map.regions[Vu2::new(0,0)].region_distances[dst]);
    // NOTE: It's okay if region_distances are based on a single direction.
    // in actual navigation algo can look at the distnaces to end of the neighbors and pick 
    // one randomly if they are the same instead!
    assert_eq!(map.regions[Vu2::new(0,0)].region_distances[dst], RegionSetDistances::Set(RegionDistances{
        left: None,
        right: Some(44),
        up: Some(40),
        down: None,
    }));
}

#[test]
fn test_region_map_hypothetical_blocks() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 7;
    let ylen:usize = 5;
    let blocked_hard = vec![
        //Vu2::new(0,0), Vu2::new(xlen as i32 -1,ylen as i32 -1), Vu2::new(xlen as i32 -1,0), Vu2::new(0, ylen as i32 - 1), 
        Vu2::new(0,0), Vu2::new(1,0), Vu2::new(2,0), Vu2::new(3,1), Vu2::new(4,2), Vu2::new(1,1), Vu2::new(2,2)];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , true, true, false, true);
    let leftv = Vu2::new(0,2 );
    let rightv = Vu2::new(6,2 );
    let downv = Vu2::new(3,0 );
    let upv = Vu2::new(3,4 );
    let middlev = Vu2::new(3,2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);

    // THIS IS IMPORTANT PART OF THE TEST!
    assert_eq!(r.get_if_will_not_cause_blocked_paths(Vu2::new(5,3)), false);
    assert_eq!(r.get_if_will_not_cause_blocked_paths(Vu2::new(3,2)), true);
    assert_eq!(r.get_if_will_not_cause_blocked_paths(Vu2::new(6,2)), true);

    // 2,1 is blocked indirectly
    assert_eq!(r.grid[2][1].point_distances[3][2], LocSetDistance::Blocked);
    
    // because of blocked stuff, distance to 1,0 is 12 for lowerright corner
    assert_eq!(r.grid[3][0].point_distances[0][1], LocSetDistance::Set(12));

    // make sure all exits are correctly set
    // also make sure all blocked are blocked
    for x in 0..7 {
        assert_eq!(r.grid[x][4].point_distances[3][2], LocSetDistance::Blocked);
        assert_eq!(r.grid[x][4].creatures.get_if_blocked(), true);
        if x == 0 {
            assert_eq!(r.grid[x][4].is_exit, ExitPoint::LeftUp);
            assert_eq!(r.grid[x][0].is_exit, ExitPoint::LeftDown);
        } else if x == xlen - 1 {
            assert_eq!(r.grid[x][4].is_exit, ExitPoint::RightUp);
            assert_eq!(r.grid[x][0].is_exit, ExitPoint::RightDown);
        } else {
            assert_eq!(r.grid[x][4].is_exit, ExitPoint::Up);
            assert_eq!(r.grid[x][0].is_exit, ExitPoint::Down);
        }
    }
    for y in 0..ylen { 
        if y !=0 && y!= ylen-1 {
            assert_eq!(r.grid[0][y].is_exit, ExitPoint::Left);
            assert_eq!(r.grid[xlen-1][y].is_exit, ExitPoint::Right);
        }
    }
    for v in &blocked_hard {
        let xx = v.x as usize;
        let yy = v.y as usize;
        assert_eq!(r.grid[xx][yy].creatures.get_if_blocked(), true);
        assert_eq!(r.grid[xx][yy].point_distances[3][2], LocSetDistance::Blocked);
    }

    // TODO: Make sure the distances to exits are correct
    println!("d d {:#?}", r.distances_from_down);
    println!("d u {:#?}", r.distances_from_up);
    println!("d l {:#?}", r.distances_from_left);
    println!("d r {:#?}", r.distances_from_right);

    assert_eq!(r.distances_from_down, InnerExitRegionDistance::Set(RegionDistances{
        left: Some(11),
        right: Some(5),
        up: None,
        down:Some(0),
    }));
    assert_eq!(r.distances_from_up, InnerExitRegionDistance::Set(RegionDistances{
        left: None,
        right: None,
        up: Some(0),
        down:None,
    }));
    assert_eq!(r.distances_from_left, InnerExitRegionDistance::Set(RegionDistances{
        left: Some(0),
        right: Some(8),
        up: None,
        down:Some(11),
    }));
    assert_eq!(r.distances_from_right, InnerExitRegionDistance::Set(RegionDistances{
        left: Some(8),
        right: Some(0),
        up: None,
        down:Some(5),
    }));
}


#[test]
#[should_panic]
fn test_invalid_regions_1() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 5;
    let ylen:usize = 7;
    let blocked_hard = vec![
        Vu2::new(2, 0)
        ];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , false, true, true, true);
    let leftv = Vu2::new(0,ylen/2 );
    let rightv = Vu2::new(xlen-1,ylen/2 );
    let downv = Vu2::new(xlen/2,0 );
    let upv = Vu2::new(xlen/2,ylen-1 );
    let middlev = Vu2::new(xlen/2,ylen/2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);
}

#[test]
fn test_invalid_regions_2() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 5;
    let ylen:usize = 7;
    let blocked_hard = vec![
        Vu2::new(0, ylen-1), Vu2::new(1, ylen-1), Vu2::new(2, ylen-1), Vu2::new(3, ylen-1)
        ];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , false, true, true, true);
    let leftv = Vu2::new(0,ylen/2 );
    let rightv = Vu2::new(xlen-1,ylen/2 );
    let downv = Vu2::new(xlen/2,0 );
    let upv = Vu2::new(xlen/2,ylen-1 );
    let middlev = Vu2::new(xlen/2,ylen/2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);
}

#[test]
#[should_panic]
fn test_invalid_regions_3() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 5;
    let ylen:usize = 7;
    let blocked_hard = vec![
        Vu2::new(1, ylen-1), Vu2::new(2, ylen-1), Vu2::new(3, ylen-1)
    ];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , true, true, true, true);
    let leftv = Vu2::new(0,ylen/2 );
    let rightv = Vu2::new(xlen-1,ylen/2 );
    let downv = Vu2::new(xlen/2,0 );
    let upv = Vu2::new(xlen/2,ylen-1 );
    let middlev = Vu2::new(xlen/2,ylen/2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);
}

#[test]
#[should_panic]
fn test_invalid_regions_4() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 5;
    let ylen:usize = 7;
    let blocked_hard = vec![
        Vu2::new(0, ylen-1), Vu2::new(1, ylen-1), Vu2::new(2, ylen-1), Vu2::new(3, ylen-1), Vu2::new(4, ylen-1)
    ];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , true, true, true, true);
    let leftv = Vu2::new(0,ylen/2 );
    let rightv = Vu2::new(xlen-1,ylen/2 );
    let downv = Vu2::new(xlen/2,0 );
    let upv = Vu2::new(xlen/2,ylen-1 );
    let middlev = Vu2::new(xlen/2,ylen/2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);
}

#[test]
#[should_panic]
fn test_invalid_regions_5() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 5;
    let ylen:usize = 7;
    let blocked_hard = vec![
        Vu2::new(xlen-1, ylen/2)
    ];
    let r = MapRegion::new(Vu2::new(1,1), xlen,ylen, 0, &blocked_hard
        , true, true, true, true);
    let leftv = Vu2::new(0,ylen/2 );
    let rightv = Vu2::new(xlen-1,ylen/2 );
    let downv = Vu2::new(xlen/2,0 );
    let upv = Vu2::new(xlen/2,ylen-1 );
    let middlev = Vu2::new(xlen/2,ylen/2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("upv: {:?}", upv);
    println!("{}", r.get_distance_strings(&upv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    
    println!("Printing whole region");
    println!("{}", r);
}

