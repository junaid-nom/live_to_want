extern crate rayon;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::{rc::Rc, cell::RefCell};
use std::io::{BufRead, BufReader};

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
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test".to_string(),
            password: "poop".to_string(),
        }), 0)).unwrap();
        
        stream.write(&wrap_ser_message(GameMessage::StringMsg("Hello There!".to_string()), 0)).unwrap();
        println!("Sent Hello  From Client");

        let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
        let mut data = String::new();
        match stream_reader.read_line(&mut data) {
            Ok(_) => {
                //let msg = &data[0..n];
                data.pop();
                let message: GameMessageWrap = serde_json::from_str(&data).unwrap();
                println!("Client got msg: {:?}", message);

                stream.write(&wrap_ser_message(GameMessage::StringMsg("Hello There2!".to_string()), 0)).unwrap();
                println!("Sent hello reply to server from client");
            },
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }
    }));

    loop {
        let msgs = server.get_messages();
        if msgs.len() > 0 {
            assert_eq!(msgs[0].username, "test".to_string());
            match &msgs[0].message {
                GameMessage::StringMsg(m) => {
                    assert_eq!(m, &"Hello There!".to_string());
                    println!("Got hello there message: {:?}", m);
                },
                _ => panic!("Should get string message!")
            }
            println!("sending msg hello reply");
            server.send_message(GameMessage::StringMsg("ServerMsg".to_string()), msgs[0].username.clone());
            break;
        }
    }
    
    loop {
        let msgs = server.get_messages();
        if msgs.len() > 0 {
            assert_eq!(msgs[0].username, "test".to_string());
            match &msgs[0].message {
                GameMessage::StringMsg(m) => assert_eq!(m, &"Hello There2!".to_string()),
                _ => panic!("Should get string message!")
            }
            break;
        }
    }
    

    // TODO NEXT: Upgrade above test to also check the client receives messages properly.
    // client receive -> replies -> check the 2nd client msg.

    //test send_message_all
    
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

#[test]
fn test_serde2() {
    let str_msg1 = "Hello There!".to_string();
    let msg1 = GameMessage::StringMsg(str_msg1.clone());
    let msg1 = GameMessageWrap{
        message: msg1,
        conn_id: 0,
    };
    let cmd_msg2 = CreatureCommandUser::Attack(1,2);
    let msg2 = GameMessage::CreatureCommandMsg(cmd_msg2.clone());
    let msg2 = GameMessageWrap{
        message: msg2,
        conn_id: 0,
    };

    let msg1_ser = serde_json::to_vec(&msg1).unwrap();
    let msg2_ser = serde_json::to_vec(&msg2).unwrap();

    let msg1_de: GameMessageWrap = serde_json::from_slice(&msg1_ser).unwrap();
    let msg2_de: GameMessageWrap = serde_json::from_slice(&msg2_ser).unwrap();

    if let GameMessage::StringMsg(msg_str) = msg1_de.message {
        assert_eq!(msg_str, str_msg1);
    } else {
        panic!("Wrong msg type");
    }
    if let GameMessage::CreatureCommandMsg(cmd_msg) = msg2_de.message {
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