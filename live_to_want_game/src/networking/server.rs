use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use std::thread::{self, JoinHandle};
use crate::{ConnectionMessageWrap, GameMessage, GameMessageWrap, UID, get_id};

pub const IP_PORT: &str = "127.0.0.1:7726";

#[tokio::main]
async fn start_server(ip_port: String, send_to_server: Sender<GameMessageWrap>, clients_sender: UnboundedSender<ConnectionMessageWrap>, mut clients_receive: UnboundedReceiver<ConnectionMessageWrap>) -> Result<(), Box<dyn std::error::Error>>  {
    // Note can't use &'static str reference because later when you move it in a thread it will complain. must be owned
    let listener = TcpListener::bind(ip_port.clone()).await?;
    
    println!("Listening on {:?} {:?}", ip_port, listener.local_addr());
    let mut started = false;
    
    let all_send_to_server = send_to_server.clone();
    tokio::spawn(async move {
        let mut client_connections: HashMap<UID, UnboundedSender<GameMessageWrap>> = HashMap::new();
        loop {
            match clients_receive.recv().await {
                Some(conn_msg) => {
                    match conn_msg {
                        ConnectionMessageWrap::SaveClientConnection(c_uid, sender) => {client_connections.insert(c_uid, sender);},
                        ConnectionMessageWrap::GameMessageWrap(game_msg_wrap) => {
                            if client_connections.contains_key(&game_msg_wrap.conn_id) {
                                match client_connections[&game_msg_wrap.conn_id].send(GameMessageWrap{
                                    conn_id: game_msg_wrap.conn_id,
                                    message: game_msg_wrap.message
                                }) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("failed to write to client channel (client dc?); err = {:?}", e);
                                        let result = all_send_to_server.send(GameMessageWrap{
                                            message: GameMessage::DropConnection(game_msg_wrap.conn_id),
                                            conn_id: 0,
                                        });
                                        match result {
                                            Ok(_) => {},
                                            Err(e) => {
                                                eprintln!("failed to write to server channel (server dc?); err = {:?}", e);
                                            },
                                        }
                                        client_connections.remove(&game_msg_wrap.conn_id);
                                    },
                                }
                            }
                        },
                    }
                },
                None => {
                    eprintln!("failed to read to clients receiver!");
                    return
                },
            }

        }
    });

    loop {
        if !started {
            send_to_server.send(string_to_game_msg("Starting Server!".to_string(), 0)).unwrap();
            started = true;
        }
        let (socket, _) = listener.accept().await?;
        let (socket_read, mut socket_write) = socket.into_split();
        let mut socket_read = BufReader::new(socket_read);
        let client_uid = get_id();
        let thread_sender_server = send_to_server.clone();
        let  (send_to_client, mut receive_client): (UnboundedSender<GameMessageWrap>, UnboundedReceiver<GameMessageWrap>) = mpsc::unbounded_channel();
        clients_sender.send(ConnectionMessageWrap::SaveClientConnection(client_uid, send_to_client)).unwrap();

        // make one thread for sending msgs to client, another to recv them from the client
        tokio::spawn(async move {
            loop {
                let msg_to_send = receive_client.recv().await;
                match msg_to_send {
                    Some(m) => {
                        if let GameMessage::DropConnection(_) = m.message {
                            socket_write.shutdown().await.unwrap();
                            return;
                        }
                        let mut serialized_m = serde_json::to_vec(&m).unwrap();
                        serialized_m.push(b'\n');
                        match socket_write.write(&serialized_m).await {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                //socket_write.shutdown().await.unwrap();
                                // I think shutdown unneccesary because we are dropping the thread here anyway
                                return;
                            },
                        }
                    },
                    None => break,
                }
            }
        });
        tokio::spawn(async move {
            let mut buf = String::new();
            // In a loop, read data from the socket and write the data to the server thread.
            loop {
                let _ = match socket_read.read_line(&mut buf).await {
                //let _ = match socket_read.read_to_end(&mut bufu8).await {
                    // socket closed
                    Ok(n) if n == 0 => {
                        println!("Got n == 0 in socket read (empty)");
                        buf.clear();
                        return
                    },
                    Ok(n) => n,
                    Err(_) => {
                        buf.clear();
                        return;
                    }
                };

                //println!("Got message from client {} : {:?}", client_uid, std::str::from_utf8(msg).unwrap());

                // Note we MUST put newline because TCP sucks and you need to separate
                // msgs yourself: https://users.rust-lang.org/t/beginner-reading-tcpstream-messages/28048
                // Serde is too dumb to read in two messgaes at once like: "{validJSON}{validJSON}"
                // super annoying. But good to seperate by newlines anyway.
                // So need to pop the newline at the end that separates files or serde is mad.
                buf.pop();
                let message = serde_json::from_str::<GameMessageWrap>(&buf);

                match message {
                    Ok(mut m) => {
                        m.conn_id = client_uid;
                        thread_sender_server.send(m).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Could not deserialize message: {:#?} err: {}", buf, e);
                    }
                };
                buf.clear();

                // https://docs.rs/serde_json/1.0.36/serde_json/de/struct.StreamDeserializer.html
                // Handles multiple messages? But it doesn't work because it doesn't separate
                // the messages.
                // let messages: Vec<GameMessageWrap> = serde_json::Deserializer::from_str(&buf)
                // .into_iter::<GameMessageWrap>().filter_map(|m| {
                //     match m {
                //         Ok(m) => Some(m),
                //         Err(e) => {
                //             eprintln!("Could not deserialize msg on server: {:#?} buf: {:#?}", e, buf);
                //             None
                //         }
                //     }
                // }).collect();
                // for mut message in messages {
                //     message.conn_id = client_uid;
                //     thread_sender_server.send(message).unwrap();
                // }
            }
        });
    }
}

pub async fn create_server(ip_port: String) -> (UnboundedSender<ConnectionMessageWrap>, Receiver<GameMessageWrap>) {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let  (sender_to_server, receive_server) = std::sync::mpsc::channel();
    let  (sender_to_clients, receieve_clients) = mpsc::unbounded_channel();
    let send_clients_clone = sender_to_clients.clone();
    thread::spawn(|| {
        start_server(ip_port, sender_to_server, send_clients_clone, receieve_clients).unwrap();
    });

    let start_msg = receive_server.recv();
    println!("Got Server start msg: {:?}", start_msg.unwrap());
    (sender_to_clients, receive_server)
}

pub fn wrap_ser_message(message: GameMessage, conn_id: UID) -> Vec<u8> {
    let msg = GameMessageWrap{
        message,
        conn_id
    };
    let mut msg = serde_json::to_vec(&msg).unwrap();
    // Note we MUST put newline because TCP sucks and you need to separate
    // msgs yourself: https://users.rust-lang.org/t/beginner-reading-tcpstream-messages/28048
    // Serde is too dumb to read in two messgaes at once like: "{validJSON}{validJSON}"
    // super annoying. But good to seperate by newlines anyway.
    msg.push(b'\n');
    
    return msg
}

pub fn string_to_game_msg(string_msg: String, conn_id: UID) -> GameMessageWrap {
    let message = GameMessage::StringMsg(string_msg);
    return GameMessageWrap{
        message,
        conn_id
    };
}

pub fn string_to_msg_buffer(string_msg: String) -> Vec<u8> {
    let msg = GameMessage::StringMsg(string_msg);
    let msg = serde_json::to_vec(&msg).unwrap();
    return msg;
}

pub fn test_client_just_print(ip_port: String) {
    match TcpStream::connect(ip_port.clone()) {
        Ok(mut stream) => {
            println!("Successfully connected to server in {}", ip_port);

            //let msg = b"Hello There!";
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
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Client Terminated.");
}

pub fn test_client_with_func(ip_port: String, f: Box<dyn Fn(TcpStream) -> () + Send> ) {
    thread::spawn(move || {
        match TcpStream::connect(ip_port.clone()) {
            Ok(stream) => {
                println!("Successfully connected to server in {}", ip_port);
                f(stream);
            },
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        println!("Client Func ended.");
    });
}

pub fn test_client_with_func_handle(ip_port: String, f: Box<dyn Fn(TcpStream) -> () + Send> ) -> JoinHandle<()> {
    let ret = thread::spawn(move || {
        let mut connected = false;
        while !connected {
            connected = match TcpStream::connect(ip_port.clone()) {
                Ok(stream) => {
                    println!("Successfully connected to server in {}", ip_port);
                    f(stream);
                    true
                },
                Err(e) => {
                    println!("Failed to connect: {}", e);
                    false
                }
            };
        }
        println!("Client Func ended.");
    });
    return ret;
}

#[tokio::main]
pub async fn dumb_server(ip_port: String) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(ip_port).await?;
    println!("Started dumb server");
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => {
                        return;
                    }
                };
                println!("Got data! {:?}", &buf[0..n]);
                
                // Write the data back
                if let Err(_) = socket.write_all(&buf[0..n]).await {
                    return;
                }
            }
        });
    }
}

pub async fn create_server_dumb(ip_port: String) -> std::sync::mpsc::Receiver<GameMessage> {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let (_, receiver) = std::sync::mpsc::channel();

    let _ = thread::spawn(|| {
        dumb_server(ip_port).unwrap();
    });
    
    println!("b4 spawn");
    // let handle = tokio::task::spawn_blocking(|| {
    //     //dumb_server().unwrap();
    //     println!("Started other thread");
    //     for i in 0..10 {
    //         // sleep(Duration::from_millis(1000));
    //         thread::sleep(time::Duration::from_millis(1000));
    //         println!("next {}", i);
    //     }
    // });
    println!("af spawn");
    //println!("waiting for start msg");
    //let start_msg = receiver.recv();
    //println!("Got start msg: {:?}", start_msg.unwrap());
    receiver
}
