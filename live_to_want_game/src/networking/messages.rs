use std::{collections::HashMap, sync::mpsc::{self, Receiver}, task::Context};

use crate::{CreatureCommandUser, GameState, UID, create_server};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameMessage {
    StringMsg(String),
    GameStateMsg(GameState),
    CreatureCommandMsg(CreatureCommandUser),
    LoginMsg(User),
    LoginReplyMsg(bool, String),
    DropConnection(UID),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameMessageWrap{
    pub message: GameMessage,
    pub conn_id: UID,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameMessageWrapUsername{
    pub message: GameMessage,
    pub username: String,
}

#[derive(Debug)]
pub enum ConnectionMessageWrap {
    SaveClientConnection(UID, UnboundedSender<GameMessageWrap>),
    GameMessageWrap(GameMessageWrap)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String
}

#[derive(Debug)]
pub struct LoginManager {
    conn_to_user: HashMap<UID, User>,
    username_to_conn: HashMap<String, UID>,
    username_to_password: HashMap<String, String>,
}
impl LoginManager {
    pub fn new() -> Self{
        LoginManager {
            conn_to_user: HashMap::new(),
            username_to_conn: HashMap::new(),
            username_to_password: HashMap::new(),
        }
    }

    // If user already logged in, but its valid, drop old connection
    pub fn login_user(&mut self, to_add: User, conn_id: UID) -> Vec<GameMessageWrap> {
        //TODO: Hash passwords
        println!("Checking pw for {}", to_add.username);
        if self.username_to_password.contains_key(&to_add.username) {
            println!("Logged in already {}", to_add.username);
            if self.username_to_password[&to_add.username] != to_add.password {
                println!("wrong pw {}", to_add.username);
                return vec![
                    GameMessageWrap{
                        message: GameMessage::LoginReplyMsg(false, "Wrong Password".to_string()),
                        conn_id: conn_id
                    }
                ];
            }
        } else {
            self.username_to_password.insert(to_add.username.clone(), to_add.password.clone());
        }

        // if logged in already... disconnect old conn?
        let mut ret = vec![
            GameMessageWrap{
                message: GameMessage::LoginReplyMsg(true, "".to_string()),
                conn_id: conn_id
            }
        ];
        if self.username_to_conn.contains_key(&to_add.username) {
            ret.push(GameMessageWrap{
                message: GameMessage::DropConnection(self.username_to_conn[&to_add.username]),
                conn_id: self.username_to_conn[&to_add.username],
            });
        }
        self.username_to_conn.insert(to_add.username.clone(), conn_id);
        self.conn_to_user.insert(conn_id, to_add);
        ret
    }

    pub fn remove_connection(&mut self, conn_id: UID) {
        let rmed = self.conn_to_user.remove(&conn_id);
        if let Some(user) = rmed {
            self.username_to_conn.remove(&user.username);
        }
    }

    pub fn get_conn_id(&self, username: &String) -> Option<UID> {
        self.username_to_conn.get(username).map_or(None, |c| Some(c.clone()))
    }

    pub fn get_all_conn_id(&self) -> Vec<UID> {
        self.username_to_conn.values().map(|c| c.clone()).collect()
    }

    pub fn get_username(&self, conn_id: &UID) -> Option<String> {
        self.conn_to_user.get(&conn_id).map_or(None, |c| Some(c.username.clone()))
    }
}

#[derive(Debug)]
pub struct ConnectionManager {
    send_to_clients: UnboundedSender<ConnectionMessageWrap>,
    receive_server_messages: Receiver<GameMessageWrap>,
    login_manager: LoginManager,
}
impl ConnectionManager {
    // on init, creates a server. Then saves the returned channels.
    // then has methods, receive message and send message.
    // it should manage username:conn_uid and stuff through login_manager
    pub async fn new(ip_port: String) -> Self {
        let (send_to_clients, receive_server_messages) = create_server(ip_port).await;
        ConnectionManager {
            send_to_clients,
            receive_server_messages,
            login_manager: LoginManager::new(),
        }
    }

    pub fn get_connected_count(&self) -> usize {
        self.login_manager.get_all_conn_id().len()
    }

    pub fn send_message(&mut self, message: GameMessage, username: String) {
        let conn_id = self.login_manager.get_conn_id(&username);
        if let Some(conn_id) = conn_id {
            match self.send_to_clients.send(ConnectionMessageWrap::GameMessageWrap(GameMessageWrap{
                message,
                conn_id
            })) {
                Ok(_) => (),
                Err(e) => eprintln!("Could not send msg to {}, err: {:#?}", username, e),
            }
        }
    }

    pub fn send_message_all(&mut self, message: GameMessage) {
        self.login_manager.get_all_conn_id().iter().for_each(|conn_id| {
            self.send_to_clients.send(ConnectionMessageWrap::GameMessageWrap(GameMessageWrap{
                message: message.clone(),
                conn_id: *conn_id
            })).unwrap();
        });
    }

    pub fn process_logins_and_get_messages(&mut self) -> Vec<GameMessageWrapUsername> {
        // Read all the current messages on the list.
        // if any are UserLogin, check if valid, if invalid respond with error.
        // if any DropConnection messages, handle here, just remove_connection from LoginManager
        let mut msgs = vec![];
        loop {
            let new_msg = self.receive_server_messages.try_recv();
            match new_msg {
                Ok(m) => {
                    match m.message {
                        GameMessage::LoginMsg(user) => {
                            // check login. if success send success msg, if not send failure msg
                            let replies = self.login_manager.login_user(user, m.conn_id);
                            // drop connection by just sending it to client which will shutdown the client socket on our end
                            replies.into_iter().for_each(|m| {
                                self.send_to_clients.send(ConnectionMessageWrap::GameMessageWrap(m)).unwrap();
                            });
                        },
                        GameMessage::DropConnection(conn_id) => {
                            self.login_manager.remove_connection(conn_id);
                        },
                        _ => {
                            msgs.push(m);
                        }
                    }
                },
                Err(e) => {
                    match e {
                        mpsc::TryRecvError::Empty => break,
                        mpsc::TryRecvError::Disconnected => panic!("Channel between server and game destroyed!"),
                    }
                },
            }
        }
        let mut ret = vec![];
        msgs.into_iter().for_each(|m| {
            let username = self.get_username(&m.conn_id);
            if let Some(username) = username {
                ret.push(GameMessageWrapUsername{
                    message: m.message,
                    username,
                });
            }
        });
        ret
    }

    pub fn get_conn_id(&self, username: &String) -> Option<UID> {
        self.login_manager.get_conn_id(username)
    }

    pub fn get_username(&self, conn_id: &UID) -> Option<String> {
        self.login_manager.get_username(conn_id)
    }
}
