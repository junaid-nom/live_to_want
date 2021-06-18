extern crate rayon;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;
use tokio::time::sleep;
use tokio::time::Duration;

use std::{thread, time};
#[test]
fn run_simple_server_test() {
    let receiver = create_server();
    
    let client_sent = 10;
    (0..client_sent).into_par_iter().for_each(|i| {
        println!("starting client {}", i);
        test_client();
    });

    (0..client_sent).into_iter().for_each(|i| {
        let msg = receiver.recv().unwrap();
        println!("Received Msg {} {:?}", i, msg);
    });
    
    // keep the server alive for awhile...
    // (0..10).into_iter().for_each(|i| {
    //     let result = receiver.recv();
    //     println!("{} Recv result: {:?}",i, result);
    // });
}

#[test]
fn test_serde() {
    let str_msg1 = "Hello There!".to_string();
    let msg1 = GameMessage::StringMsg(str_msg1.clone());
    let cmd_msg2 = CreatureCommandUser::Attack(1,2);
    let msg2 = GameMessage::CreatureCommandMsg(cmd_msg2.clone());

    let msg1_ser = serde_json::to_vec(&msg1).unwrap();
    let msg2_ser = serde_json::to_vec(&msg2).unwrap();

    let msg1_de: GameMessage = serde_json::from_slice(&msg1_ser).unwrap();
    let msg2_de: GameMessage = serde_json::from_slice(&msg2_ser).unwrap();

    if let GameMessage::StringMsg(msg_str) = msg1_de {
        assert_eq!(msg_str, str_msg1);
    } else {
        panic!("Wrong msg type");
    }
    if let GameMessage::CreatureCommandMsg(cmd_msg) = msg2_de {
        assert_eq!(cmd_msg, cmd_msg2);
    } else {
        panic!("Wrong msg type");
    }
}

#[tokio::test]
async fn run_dumb_server_test() {
    create_server_dumb().await;
    println!("Ran create server");
    thread::sleep(time::Duration::from_millis(1000*60*5));
}