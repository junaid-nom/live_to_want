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

// TODONEXT: Eventually when game loop actually uses a server, make a basic test that
// Login in -> Creates player character -> moves player character around
// Also test logging in and out 
// Eventually need to also make a scenario where the player attacks and kills something.

#[tokio::test]
async fn run_simple_server_test() {
    let ip_port: String = "127.0.0.1:7727".to_string();

    // - One test that uses ConnectionManager to create a server.
    // Then in a loop call get_msgs every x seconds.
    // Also in another thread call test_clients that try to login and then send
    // a string message. make sure the string message arrives with the right username

    let mut server = ConnectionManager::new(ip_port.clone()).await;
    test_client_with_func(ip_port.clone(), Box::new(|mut stream: TcpStream| {
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

    test_client_with_func(ip_port.clone(), Box::new(|mut stream: TcpStream| {
        stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
            username: "test2".to_string(),
            password: "poop".to_string(),
        }), 0)).unwrap();

        stream.write(&wrap_ser_message(GameMessage::StringMsg("Hello There2!".to_string()), 0)).unwrap();
    }));

    let mut got_test = false;
    let mut got_test2 = false;
    while !got_test || !got_test2 {
        let msgs = server.process_logins_and_get_messages();
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
    
    test_client_with_func(ip_port.clone(), Box::new(|mut stream: TcpStream| {
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
        let msgs = server.process_logins_and_get_messages();
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
    println!("GOT IT!");
}


#[tokio::test]
async fn run_connection_manager_test_send_all_and_multi_login() {
    let ip_port: String = "127.0.0.1:7728".to_string(); // make sure port is diff so tests can run concurrently

    // test many clients connected.
    // have one try to login to same user already connected.
    // make sure first connector is forcefully dced.
    // then send_all to make sure stuff is working.

    let mut server = ConnectionManager::new(ip_port.clone()).await;
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
        test_client_with_func(ip_port.clone(), make_client_func(format!("u{}", i)));
    });

    let mut got_msgs = 0;
    while got_msgs != clients_started {
        let msgs = server.process_logins_and_get_messages();
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

    test_client_with_func(ip_port.clone(), make_overwrite_client_func(format!("u9")));

    server.send_message_all(GameMessage::StringMsg("ServerMsg".to_string()));

    // should get (clients_started - 1) h2s and 1 from u9 thats u9
    let mut got_msgs = 0;
    while got_msgs != clients_started {
        let msgs = server.process_logins_and_get_messages();
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
    let ip_port: String = "127.0.0.1:7729".to_string(); // make sure port is diff so tests can run concurrently

    create_server_dumb(ip_port).await;
    println!("Ran create server");
    // I think I used the below to test in my browser/postman if stuff is working.
    //thread::sleep(time::Duration::from_millis(1000*60*5));
    thread::sleep(time::Duration::from_millis(1000));
}

#[tokio::test]
async fn run_game_server() {
    let ip_port: String = "127.0.0.1:7730".to_string(); // make sure port is diff so tests can run concurrently

    let openr = RegionCreationStruct::new(10,10, 0, vec![]);
    let rgrid = vec![
        vec![openr.clone()],
    ];
    //create map
    let mut map = MapState::new(rgrid, 0);
    let  region: &mut MapRegion = &mut map.regions[0][0];

    let mut grass = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    grass.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    grass.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    grass.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    grass.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Grass,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    grass.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    grass.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut flower = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    flower.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    flower.components.location_component = LocationComponent {
        location: Vu2{x: 1, y: 1}
    };
    flower.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    flower.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Flower,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    flower.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    flower.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut bush = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    bush.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    bush.components.location_component = LocationComponent {
        location: Vu2{x: 2, y: 1}
    };
    bush.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    bush.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::Bush,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    bush.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 1, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    bush.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });

    let mut tree = CreatureState{
        components: ComponentMap::default(),
        inventory: Vec::new(),
        memory: CreatureMemory::default(),
    };
    tree.components.region_component = RegionComponent {
        region: Vu2{x: 0, y: 0},
    };
    tree.components.location_component = LocationComponent {
        location: Vu2{x: 7, y: 1}
    };
    tree.components.health_component = Some(HealthComponent {
        health:  1,
        max_health: 1,
    });
    tree.components.soil_component = Some(SoilComponent{
        soil_height: SoilHeight::All,
        soil_type_cannot_grow: SoilType::Clay,
        soil_type_spread: SoilType::Sand,
        frame_ready_to_spread: 0,
        spread_rate: Some(1),
    });
    tree.components.budding_component = Some(BuddingComponent { 
        reproduction_rate: 2, frame_ready_to_reproduce: 0, seed_creature_differences: Box::new(ComponentMap::fake_default())
    });
    // Just to make sure the grass doesn't replicate with the inventory
    tree.inventory.push(Item{
        item_type: ItemType::Berry,
        quantity: 1,
    });


    let grass_loc = grass.components.location_component.location;
    region.grid[grass_loc].creatures.set_soil(SoilType::Sand);
    let flower_loc = flower.components.location_component.location;
    region.grid[flower_loc].creatures.set_soil(SoilType::Sand);
    let bush_loc = bush.components.location_component.location;
    region.grid[bush_loc].creatures.set_soil(SoilType::Sand);
    let tree_loc = tree.components.location_component.location;
    region.grid[tree_loc].creatures.set_soil(SoilType::Sand);

    region.grid[grass_loc].creatures.add_creature(
        grass, 0
    );
    region.grid[flower_loc].creatures.add_creature(
        flower, 0
    );
    region.grid[bush_loc].creatures.add_creature(
        bush, 0
    );
    region.grid[tree_loc].creatures.add_creature(
        tree, 0
    );

    // See last post of: https://users.rust-lang.org/t/how-to-use-async-fn-in-thread-spawn/46413/5
    // for how this works
    let ip_port_copy = ip_port.clone();
    let server_handle = tokio::spawn(async move {
        println!("Running server thread");
        create_game_server(ip_port_copy, map, 500, true).await;
    });
    // Note need to use tokio threads because we have a ConnectionManager.await.
    // Then need to await this server_handle or it will never run 
    // Unless we await or yield or something something else in this function.
    // To make it simpler, just await the tokio spawn which will actually
    // run the server msg loop in a different thread.
    server_handle.await.unwrap();
    
    fn make_client_func(username: String) -> Box<dyn Fn(TcpStream) -> () + Send> {
        return Box::new(move |mut stream: TcpStream| {
            println!("Sending login message");

            let username = username.clone();
            stream.write(&wrap_ser_message(GameMessage::LoginMsg(User{
                username: username.clone(),
                password: "poop".to_string(),
            }), 0)).unwrap();
            
            println!("Sent login message");

            let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
            let mut data = String::new();
            let mut finished = false;
            while !finished {
                data.clear();
                println!("Waiting for reply");
                match stream_reader.read_line(&mut data) {
                    Ok(_) => {
                        println!("Got a msg as client");
                        data.pop();
                        let message_received: GameMessageWrap = serde_json::from_str(&data).expect(&format!("Couldnt serialize {}", data));
                        if let GameMessage::LoginReplyMsg(succ, name) = message_received.message {
                            println!("Got login reply message: {} {}", succ, name);
                        } else if let GameMessage::GameStateMsg(game_state) = message_received.message {
                            println!("Got game state: frame: {}", game_state.map_state.frame_count);
                            if game_state.map_state.frame_count >= 3 {
                                println!("Got game state: {:#?}", &data);
                                finished = true;
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            };
        });
    }

    test_client_with_func_handle(ip_port, make_client_func(format!("userguy"))).join().expect("couldnt start client");
}
