extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;

#[test]
fn test_region_map_update() {
    println!("Poop test");
    let r = MapRegion::new(7,5, 0, &Vec::new(), 
        true, true, false, true);
    println!("{}", r);
    assert_eq!(false, true);
}
