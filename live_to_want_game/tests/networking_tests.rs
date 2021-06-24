extern crate rayon;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::{rc::Rc, cell::RefCell};

use rayon::prelude::*;
use live_to_want_game::*;
use tokio::time::sleep;
use tokio::time::Duration;

use std::{thread, time};

#[tokio::test]
async fn run_simple_server_test() {
    // TODONEXT: Make real server tests:

    // - One test that uses ConnectionManager to create a server.
    // Then in a loop call get_msgs every x seconds.
    // Also in another thread call test_clients that try to login and then send
    // a string message. make sure the string message arrives with the right username

    // Do simple test just to see if the server is receiving messages.
    let mut server = ConnectionManager::new().await;
    test_client_with_func(Box::new(|mut stream: TcpStream| {
        let msg = GameMessage::LoginMsg(User{
            username: "test".to_string(),
            password: "poop".to_string(),
        });
        let msg = serde_json::to_vec(&msg).unwrap();
        stream.write(&msg).unwrap();

        let msg = GameMessage::StringMsg("Hello There!".to_string());
        let msg = serde_json::to_vec(&msg).unwrap();
        stream.write(&msg).unwrap();
        println!("Sent Hello");
        //let mut data = [0 as u8; 12]; // using 6 byte buffer
        let mut data = vec![];
        match stream.read_to_end(&mut data) {
            Ok(n) => {
                let msg = &data[0..n];
                let message: GameMessageWrap = serde_json::from_slice(msg).unwrap();
                println!("Got msg: {:?}", message);
            },
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }
    }));

    let msgs = server.get_messages();
    if msgs.len() > 0 {
        assert_eq!(msgs[0].username, "test".to_string());
        match &msgs[0].message {
            GameMessage::StringMsg(m) => assert_eq!(m, &"Hello There!".to_string()),
            _ => panic!("Should get string message!")
        }
    }

    // TODO NEXT: Upgrade above test to also check the client receives messages properly.
    // client receive -> replies -> check the 2nd client msg.
    
    // in another test have a server from ConnectionManager.
    // have a test_client login.
    // then have a 2nd one login with same pw.
    // make sure first test_client is disconnected on client end.
    // dc the 2nd test client on the client end.
    // try to send a message to the DCed test_client. make sure nothing weird happens
    // make sure connectionManager drops the conn.

    // let receiver = create_server();
    
    // let client_sent = 10;
    // (0..client_sent).into_par_iter().for_each(|i| {
    //     println!("starting client {}", i);
    //     test_client();
    // });

    // (0..client_sent).into_iter().for_each(|i| {
    //     let msg = receiver.recv().unwrap();
    //     println!("Received Msg {} {:?}", i, msg);
    // });
    
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