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

    let mut server = ConnectionManager::new().await;
    test_client_with_func(Box::new(|mut stream: TcpStream| {
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test".to_string(),
            password: "poop".to_string(),
        }), 0)).unwrap();
        
        stream.write(&wrap_ser_message(GameMessage::StringMsg("Hello There\n!".to_string()), 0)).unwrap();
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

    test_client_with_func(Box::new(|mut stream: TcpStream| {
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test2".to_string(),
            password: "poop".to_string(),
        }), 0)).unwrap();

        stream.write(&wrap_ser_message(GameMessage::StringMsg("Hello There2!".to_string()), 0)).unwrap();
    }));

    let mut got_test = false;
    let mut got_test2 = false;
    while !got_test || !got_test2 {
        let msgs = server.get_messages();
        msgs.into_iter().for_each(
            |g| {
                let test_string = if g.username == "test".to_string() {
                    "Hello There\n!".to_string()
                } else { "Hello There2!".to_string() };

                if g.username == "test".to_string() {
                    got_test = true;
                }
                else if g.username == "test2".to_string() {
                    got_test2 = true;
                }

                match g.message {
                    GameMessage::StringMsg(m) => {
                        assert_eq!(m, test_string);
                        println!("Got hello there message: {:?}", m);
                    },
                    _ => panic!("Should get string message!")
                }

                server.send_message(GameMessage::StringMsg("ServerMsg".to_string()), g.username.clone());
            }
        );
    }
    
    test_client_with_func(Box::new(|mut stream: TcpStream| {
        // below should fail to login b/c wrong password
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test2".to_string(),
            password: "poop22".to_string(),
        }), 0)).unwrap();

        stream.write(&wrap_ser_message(GameMessage::StringMsg("fail".to_string()), 0)).unwrap();

        //wait for response first:
        let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
        let mut data = String::new();
        match stream_reader.read_line(&mut data) {
            Ok(_) => {},
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }

        // login correctly
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test2".to_string(),
            password: "poop".to_string(),
        }), 0)).unwrap();

        stream.write(&wrap_ser_message(GameMessage::StringMsg("yes".to_string()), 0)).unwrap();
    }));
    

    let mut got_test = false;
    let mut got_test2 = false;
    while !got_test || !got_test2 {
        let msgs = server.get_messages();
        msgs.into_iter().for_each(
            |g| {
                let test_string = if g.username == "test".to_string() {
                    "Hello There2!".to_string()
                } else { "yes".to_string() };

                if g.username == "test".to_string() {
                    got_test = true;
                }
                else if g.username == "test2".to_string() {
                    got_test2 = true;
                }

                match g.message {
                    GameMessage::StringMsg(m) => {
                        assert_eq!(m, test_string);
                        println!("Got hello there message: {:?}", m);
                    },
                    _ => panic!("Should get string message!")
                }

                server.send_message(GameMessage::StringMsg("ServerMsg".to_string()), g.username.clone());
            }
        );
    }
}


#[tokio::test]
async fn run_connection_manager_test_send_all_and_multi_login() {
    // test many clients connected.
    // have one try to login to same user already connected.
    // make sure first connector is forcefully dced.
    // then send_all to make sure stuff is working.

    let mut server = ConnectionManager::new().await;
    fn make_client_func(username: String) -> Box<dyn Fn(TcpStream) -> () + Send> {
        return Box::new(move |mut stream: TcpStream| {
            let username = username.clone();
            stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
                username: username.clone(),
                password: "poop".to_string(),
            }), 0)).unwrap();
            
            stream.write(&wrap_ser_message(GameMessage::StringMsg("h1".to_string()), 0)).unwrap();
    
            let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
            let mut data = String::new();
            for _ in 0..2 {
                data.clear();
                match stream_reader.read_line(&mut data) {
                    Ok(_) => {
                        data.pop();
                        let message_received: GameMessageWrap = serde_json::from_str(&data).expect(&format!("Couldnt serialize {}", data));
                        if let GameMessage::LoginReplyMsg(_, _) = message_received.message {
    
                        } else {
                            assert_ne!(username, "u9".to_string());
                            stream.write(&wrap_ser_message(GameMessage::StringMsg("h2".to_string()), 0)).unwrap();
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            };
        });
    }
    let clients_started = 10;
    (0..clients_started).for_each(|i| {
        test_client_with_func(make_client_func(format!("u{}", i)));
    });

    let mut got_msgs = 0;
    while got_msgs != clients_started {
        let msgs = server.get_messages();
        msgs.into_iter().for_each(
            |g| {
                match g.message {
                    GameMessage::StringMsg(m) => {
                        assert_eq!(m, "h1".to_string());
                    },
                    _ => panic!("Should get string message!")
                }
                got_msgs +=1;
            }
        );
    }

    // overwrite one user.
    fn make_overwrite_client_func(username: String) -> Box<dyn Fn(TcpStream) -> () + Send> {
        return Box::new(move |mut stream: TcpStream| {
            stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
                username: username.clone(),
                password: "poop".to_string(),
            }), 0)).unwrap();
                
            let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
            let mut data = String::new();
            match stream_reader.read_line(&mut data) {
                Ok(_) => {
                    data.pop();
                    let _message_received: GameMessageWrap = serde_json::from_str(&data).unwrap();
                    stream.write(&wrap_ser_message(GameMessage::StringMsg(username.clone()), 0)).unwrap();
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        });
    }

    test_client_with_func(make_overwrite_client_func(format!("u9")));

    server.send_message_all(GameMessage::StringMsg("ServerMsg".to_string()));

    // should get (clients_started - 1) h2s and 1 from u9 thats u9
    let mut got_msgs = 0;
    while got_msgs != clients_started {
        let msgs = server.get_messages();
        msgs.into_iter().for_each(
            |g| {
                match g.message {
                    GameMessage::StringMsg(m) => {
                        if g.username == "u9".to_string() {
                            assert_eq!(m, "u9".to_string());
                        } else {
                            assert_eq!(m, "h2".to_string());
                        }
                    },
                    _ => panic!("Should get string message!")
                }
                got_msgs +=1;
            }
        );
    }
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