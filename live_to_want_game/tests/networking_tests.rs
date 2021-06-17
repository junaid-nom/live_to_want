extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;
use std::{thread, time};

#[test]
fn run_simple_server_test() {
    //create_server_dumb();
    dumb_server().unwrap();
    println!("started server");
    thread::sleep(time::Duration::from_millis(1000*60*5));

    let receiver = create_server();
    
    thread::sleep(time::Duration::from_millis(1000*60*5));

    (0..10).into_par_iter().for_each(|i| {
        println!("starting client {}", i);
        test_client();
    });


    // keep the server alive for awhile...
    (0..10).into_iter().for_each(|i| {
        let result = receiver.recv();
        println!("{} Recv result: {:?}",i, result);
    });
}
