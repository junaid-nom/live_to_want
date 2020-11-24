extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_region_map_update() {
    println!("Poop test");
    let xlen:usize = 7;
    let ylen:usize = 5;
    let blocked_hard = vec![
        //Vector2::new(0,0), Vector2::new(xlen as i32 -1,ylen as i32 -1), Vector2::new(xlen as i32 -1,0), Vector2::new(0, ylen as i32 - 1), 
        Vector2::new(2,0), Vector2::new(3,1), Vector2::new(4,2), Vector2::new(1,1), Vector2::new(2,2)];
    let r = MapRegion::new(xlen,ylen, 0, &blocked_hard
        , true, true, false, true);
    let leftv = Vector2::new(0,4 );
    let rightv = Vector2::new(6,4 );
    let downv = Vector2::new(6,0 );
    let middlev = Vector2::new(3,2);
    println!("leftv: {:?}", leftv);
    println!("{}", r.get_distance_strings(&leftv).join("\n"));
    println!("rightv: {:?}", rightv);
    println!("{}", r.get_distance_strings(&rightv).join("\n"));
    println!("downv: {:?}", downv);
    println!("{}", r.get_distance_strings(&downv).join("\n"));
    println!("middlev: {:?}", middlev);
    println!("{}", r.get_distance_strings(&middlev).join("\n"));
    println!("Printing whole region");
    println!("{}", r);

    // 2,1 is blocked indirectly
    assert_eq!(r.grid[2][1].point_distances[3][2], LocDistance::Blocked);
    
    // because of blocked stuff, distance to 1,0 is 13 for lowerright corner
    assert_eq!(r.grid[6][0].point_distances[1][0], LocDistance::Set(13));

    // make sure all exits are correctly set
    // also make sure all blocked are blocked
    for x in 0..7 {
        assert_eq!(r.grid[x][4].point_distances[3][2], LocDistance::Blocked);
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
        assert_eq!(r.grid[xx][yy].point_distances[3][2], LocDistance::Blocked);
    }

    // TODO: Make sure the distances to exits are correct
    println!("d d {:#?}", r.distances_from_down);
    println!("d u {:#?}", r.distances_from_up);
    println!("d l {:#?}", r.distances_from_left);
    println!("d r {:#?}", r.distances_from_right);

    assert_eq!(r.distances_from_down, LocRegionDistance::Set(RegionDistances{
        left: Some(12),
        right: Some(0),
        up: None,
        down:Some(0),
    }));
    assert_eq!(r.distances_from_up, LocRegionDistance::Set(RegionDistances{
        left: None,
        right: None,
        up: Some(0),
        down:None,
    }));
    assert_eq!(r.distances_from_down, LocRegionDistance::Set(RegionDistances{
        left: Some(12),
        right: Some(0),
        up: None,
        down:Some(0),
    }));
    assert_eq!(r.distances_from_down, LocRegionDistance::Set(RegionDistances{
        left: Some(12),
        right: Some(0),
        up: None,
        down:Some(0),
    }));

    assert_eq!(true, false);
}
