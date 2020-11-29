extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_map_state() {
    
}

#[test]
fn test_region_map_hypothetical_blocks() {
    println!("test_region_map_hypothetical_blocks test");
    let xlen:usize = 7;
    let ylen:usize = 5;
    let blocked_hard = vec![
        //Vu2::new(0,0), Vu2::new(xlen as i32 -1,ylen as i32 -1), Vu2::new(xlen as i32 -1,0), Vu2::new(0, ylen as i32 - 1), 
        Vu2::new(0,0), Vu2::new(1,0), Vu2::new(2,0), Vu2::new(3,1), Vu2::new(4,2), Vu2::new(1,1), Vu2::new(2,2)];
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
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

