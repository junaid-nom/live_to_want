use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::mpsc::{self, Receiver, Sender};
use crate::GameMessage;

const IP_PORT: &str = "127.0.0.1:7726";

#[tokio::main]
pub async fn dumb_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(IP_PORT).await?;

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

async fn start_server(sender: Sender<GameMessage>) -> Result<(), Box<dyn std::error::Error>>  {
    let listener = TcpListener::bind(IP_PORT).await?;
    
    println!("Listening on {:?} {:?}", IP_PORT, listener.local_addr());
    let mut started = false;
    loop {
        if !started {
            sender.send(string_to_game_msg("Starting Server!".to_string())).unwrap();
            started = true;
            println!("sent started");
        }
        let (mut socket, _) = listener.accept().await?;
        let thread_sender = sender.clone();
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

                let msg = &buf[0..n];
                let message: GameMessage = serde_json::from_slice(msg).unwrap();
                thread_sender.send(message).unwrap();

                // Write the data back
                if let Err(e) = socket.write_all(msg).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

#[tokio::main]
pub async fn create_server() -> Receiver<GameMessage> {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let (sender, receiver) = mpsc::channel();
    tokio::spawn(async move {
        start_server(sender).await.unwrap();
    });

    println!("waiting for start msg");
    let start_msg = receiver.recv();
    println!("Got start msg: {:?}", start_msg.unwrap());
    receiver
}

#[tokio::main]
pub async fn create_server_dumb() -> Receiver<GameMessage> {
    // let (sender: Sender<GameMessage>, receiver: Receiver<GameMessage>) = mpsc::channel();
    let (sender, receiver) = mpsc::channel();
    tokio::spawn(async move {
        //dumb_server().await.unwrap();
    });

    println!("waiting for start msg");
    //let start_msg = receiver.recv();
    //println!("Got start msg: {:?}", start_msg.unwrap());
    receiver
}

pub fn string_to_game_msg(string_msg: String) -> GameMessage {
    let msg = GameMessage::StringMsg(string_msg);
    return msg;
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
            println!("Sent Hello, awaiting reply...");
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
