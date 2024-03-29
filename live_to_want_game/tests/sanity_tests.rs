use std::{sync::Mutex, cell::{Ref, RefCell}, rc::Rc, sync::Arc};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fmt::{Debug, Formatter}, borrow::Borrow};
use std::sync::atomic::AtomicU64;
use core::fmt;

extern crate rayon;
use live_to_want_game::*;
use rand::Rng;
use rayon::prelude::*;


#[test]
fn test_arc_mutex() {
    let mut up_exit = Arc::new(Mutex::new(None));
    (0..10).into_iter().for_each(
        |x| {
            let up_exit = Arc::clone(&up_exit);
            let mut up_exit = up_exit.lock().unwrap();
            match *up_exit {
                Some(a) => {*up_exit = Some(a+1)}
                None => {*up_exit = Some(0)}
            } 
        }
    );
    println!("{:#?}", up_exit);
}

#[test]
fn iter_iter_par() {
    let x = vec![vec![1,2,3],vec![1,2,3],vec![1,2,3]];
    let new: Vec<i32> = x.par_iter().flat_map(|x| {
        let r: Vec<i32> = x.par_iter().map(|y| {
            y+1
        }).collect();
        r
    }).collect();
}


#[test]
fn reality_exists() {
    assert_eq!(2 + 2, 4);
}
#[test]
#[should_panic]
fn how_to_rc_refcell() {
    let r = Rc::new(RefCell::new(Vector2{x: 0, y:0}));
    let mut r2 = r.deref().borrow_mut();
    r2.x = 5;
    let mut d = r.deref().borrow_mut();
    d.x = 6;
    r2.x = 10;
    //assert_eq!(r.clone().deref().borrow_mut().x, 10);
}

#[test]
fn how_mut_ref_works() {
    fn my_mut(loc: &mut Vector2) {
        loc.x +=1;
        if loc.x < 10 {
            my_mut(loc);
        }
        loc.x +=1;
    }
    let mut loc = Vector2{x:0, y:0};
    my_mut(&mut loc);
    loc.x -= 5;
    my_mut(&mut loc);
    loc.y += 1;
}

#[test]
fn how_vecs_ownership_works() { 
    let mut vec1 = vec![MapState::default()];
    let mut vec2 :Vec<MapState> = Vec::new();
    let trans = vec1.remove(0);
    vec2.push(trans);
    assert_eq!(vec1.len() + 1, vec2.len());
}

#[test]
fn how_does_mut_ref_work() {
    fn need_immutable(loc: &Vector2) -> i32 {
        loc.x
    }
    fn need_mutable(loc: &mut Vector2) -> i32 {
        loc.x += 1;
        loc.x
    }

    let mut loc = Vector2{x: 1, y:2};
    let loc_m = &mut loc;
    need_immutable(loc_m);
    need_mutable(loc_m);
    need_immutable(loc_m);
    need_mutable(loc_m);
    assert_eq!(loc.x, 3);
}

#[test]
fn how_does_mut_state_work_nested_obj() {
    struct MutMl<'a> {
        ml: &'a mut MapLocation,
    }

    fn use_ml(ml: &MapLocation) -> usize {
        ml.location.x
    }
    fn change_ml(ml: &mut Vu2) {
        ml.x += 1;
    }

    let mut ml = MapLocation::default();

    let mml = MutMl{
        ml: &mut ml,
    };
    // both of below won't work!
    
    // let mml2 = MutMl{
    //     ml: &mut ml,
    // };
    //use_ml(&ml);
}

#[test]
fn graph_without_vec_test() {
    pub struct Node<'a> {
        children: Vec<&'a Node<'a>>,
        my_num: u32,
    }
    impl Node<'_> {
        fn new<'a>(num: u32) -> Node<'a> {
            Node{
                children: Vec::new(),
                my_num: num,
            }
        }
    }
    pub struct NodeRoot<'a> {
        root: Node<'a>,
        left: Node<'a>,
        right: Node<'a>,
        child_both: Node<'a>,
        child_left: Node<'a>,
    }
    pub struct NodeWrapper<'a> {
        root_graph: NodeRoot<'a>,
    }

    fn make_node<'a>() -> NodeRoot<'a> {
        let mut node_root = NodeRoot {
            root: Node::new(0),
            left: Node::new(1),
            right: Node::new(2),
            child_both: Node::new(3),
            child_left: Node::new(4),
        };
        // need unsafe to self reference and return something
        unsafe {
            node_root.right.children.push(std::mem::transmute(&node_root.child_both));
            node_root.left.children.push(std::mem::transmute(&node_root.child_both));
            node_root.left.children.push(std::mem::transmute(&node_root.child_left));
            node_root.root.children.push(std::mem::transmute(&node_root.left));
            node_root.root.children.push(std::mem::transmute(&node_root.right));
        }
        
        node_root
    }
    let mut root = make_node();
    let new_c = Node::new(5);
    root.child_both = new_c;
    let wrap = NodeWrapper {
        root_graph: root,
    };
    let wrap2 = Box::new(wrap);

    assert_eq!(wrap2.deref().root_graph.root.children[0].children[0].my_num,5);
    assert_eq!(wrap2.deref().root_graph.root.children[0].children[1].my_num,4);
    // TODO: NOT SURE HOW TO BREAK THIS? But apparently it can be broken and is unsafe?
}

#[test]
fn bool_list_contains() {
    let v1 = vec![false, false, true, false];
    let v2 = vec![false, false, false, false];

    assert_eq!(v1.contains(&true), true);
    assert_eq!(v2.contains(&true), false);
}


#[test]
fn test_unsigned_subtraction() {
    let mut rng = rand::thread_rng();
    let r: u32 = rng.gen_range(500, 1000);
    let v100: u32 = 100;

    // this causes panic //assert!(v100 - r > 0); // overflows into wrap as default
    assert!(v100.saturating_sub(r) == 0);
}

#[test]
fn test_closer_mut() {
    let mut avg = 0;
    let mut add_avg = |rd: &Option<u32>| match rd {
        Some(d) => {avg += d;}
        None => {}
    };
    let x1 = Some(2);
    let x2 = Some(3);
    let x3 = Some(4);
    add_avg(&x1);
    add_avg(&x2);
    add_avg(&x3);
    assert_eq!(avg, 9);
}

#[test]
#[should_panic]
fn test_usize() {
    let mut x: usize = 5;
    x = x-6; // This causes a panic jesus
    assert_eq!(x, 0);
}

#[test]
fn test_mod() {
    assert_eq!(2 % 2, 0);
}
