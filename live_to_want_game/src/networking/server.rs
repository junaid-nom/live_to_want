use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use std::{thread, time};
use crate::{ConnectionMessageWrap, GameMessage, GameMessageWrap, UID, get_id};
use tokio::time::{sleep, Duration};

const IP_PORT: &str = "127.0.0.1:7726";

#[tokio::main]
async fn start_server(send_to_server: UnboundedSender<GameMessageWrap>, clients_sender: UnboundedSender<ConnectionMessageWrap>, mut clients_receive: UnboundedReceiver<ConnectionMessageWrap>) -> Result<(), Box<dyn std::error::Error>>  {
    let listener = TcpListener::bind(IP_PORT).await?;
    
    println!("Listening on {:?} {:?}", IP_PORT, listener.local_addr());
    let mut started = false;
    
    let all_sender_server = send_to_server.clone();
    tokio::spawn(async move {
        let mut client_connections: HashMap<UID, UnboundedSender<GameMessageWrap>> = HashMap::new();
        loop {
            match clients_receive.recv().await {
                Some(conn_msg) => {
                    match conn_msg {
                        ConnectionMessageWrap::SaveClientConnection(c_uid, sender) => {client_connections.insert(c_uid, sender);},
                        ConnectionMessageWrap::GameMessageWrap(game_msg_wrap) => {
                            if client_connections.contains_key(&game_msg_wrap.conn_uid) {
                                match client_connections[&game_msg_wrap.conn_uid].send(GameMessageWrap{
                                    conn_uid: game_msg_wrap.conn_uid,
                                    message: game_msg_wrap.message
                                }) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        eprintln!("failed to write to client channel (client dc?); err = {:?}", e);
                                        all_sender_server.send(GameMessageWrap{
                                            message: GameMessage::DropConnection(game_msg_wrap.conn_uid),
                                            conn_uid: 0,
                                        }).unwrap();
                                        client_connections.remove(&game_msg_wrap.conn_uid);
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
            println!("sent started");
        }
        let (socket, _) = listener.accept().await?;
        let (mut socket_read, mut socket_write) = socket.into_split();
        let client_uid = get_id();
        let thread_sender_server = send_to_server.clone();
        let  (send_to_client, mut receive_client): (UnboundedSender<GameMessageWrap>, UnboundedReceiver<GameMessageWrap>) = mpsc::unbounded_channel();
        clients_sender.send(ConnectionMessageWrap::SaveClientConnection(client_uid, send_to_client)).unwrap();

        // make one thread for receiving msgs from client, another to send them to the client
        tokio::spawn(async move {
            loop {
                let msg_to_send = receive_client.recv().await;
                match msg_to_send {
                    Some(m) => {
                        if let GameMessage::DropConnection(_) = m.message {
                            socket_write.shutdown().await.unwrap();
                            return;
                        }
                        let serialized_m = serde_json::to_vec(&m).unwrap();
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
            
            println!("Turning off client sending {}", client_uid);
        });
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            
            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket_read.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                let msg = &buf[0..n];
                let message: GameMessageWrap = serde_json::from_slice(msg).unwrap();

                thread_sender_server.send(message).unwrap();
            }
        });
    }
}

pub async fn create_server() -> (UnboundedSender<ConnectionMessageWrap>, UnboundedReceiver<GameMessageWrap>) {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let  (sender_to_server, mut receive_server) = mpsc::unbounded_channel();
    let  (sender_to_clients, receieve_clients) = mpsc::unbounded_channel();
    let send_clients_clone = sender_to_clients.clone();
    thread::spawn(|| {
        start_server(sender_to_server, send_clients_clone, receieve_clients).unwrap();
    });

    let start_msg = receive_server.recv().await;
    println!("Got start msg: {:?}", start_msg.unwrap());
    (sender_to_clients, receive_server)
}

pub fn string_to_game_msg(string_msg: String, conn_uid: UID) -> GameMessageWrap {
    let message = GameMessage::StringMsg(string_msg);
    return GameMessageWrap{
        message,
        conn_uid
    };
}

pub fn string_to_msg_buffer(string_msg: String) -> Vec<u8> {
    let msg = GameMessage::StringMsg(string_msg);
    let msg = serde_json::to_vec(&msg).unwrap();
    return msg;
}

pub fn test_client() {
    match TcpStream::connect(IP_PORT) {
        Ok(mut stream) => {
            println!("Successfully connected to server in {}", IP_PORT);

            //let msg = b"Hello There!";
            let msg = GameMessage::StringMsg("Hello There!".to_string());
            let msg = serde_json::to_vec(&msg).unwrap();

            stream.write(&msg).unwrap();
            println!("Sent Hello");
            // let mut data = [0 as u8; 12]; // using 6 byte buffer
            // match stream.read_exact(&mut data) {
            //     Ok(_) => {
            //         if &data == &msg {
            //             println!("Reply is ok!");
            //         } else {
            //             let text = from_utf8(&data).unwrap();
            //             println!("Unexpected reply: {}", text);
            //         }
            //     },
            //     Err(e) => {
            //         println!("Failed to receive data: {}", e);
            //     }
            // }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Client Terminated.");
}

#[tokio::main]
pub async fn dumb_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(IP_PORT).await?;
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
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                println!("Got data! {:?}", &buf[0..n]);
                
                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

pub async fn create_server_dumb() -> std::sync::mpsc::Receiver<GameMessage> {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let (sender, receiver) = std::sync::mpsc::channel();

    let handler = thread::spawn(|| {
        dumb_server().unwrap();
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
