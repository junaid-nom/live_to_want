use std::collections::HashMap;

use crate::{CreatureCommandUser, GameState, UID, create_server};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameMessage {
    StringMsg(String),
    GameStateMsg(GameState),
    CreatureCommandMsg(CreatureCommandUser),
    LoginMsg{user: User},
    DropConnection(UID),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameMessageWrap{
    pub message: GameMessage,
    pub conn_uid: UID,
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
    pub fn login_user(&mut self, to_add: User, conn_id: UID) -> Option<GameMessageWrap> {
        //TODO: Hash passwords
        if self.username_to_password.contains_key(&to_add.username) {
            if self.username_to_password[&to_add.username] != to_add.password {
                return None;
            }
        }

        // if logged in already... disconnect old conn?
        let ret = if self.username_to_conn.contains_key(&to_add.username) {
            Some(GameMessageWrap{
                message: GameMessage::DropConnection(self.username_to_conn[&to_add.username]),
                conn_uid: self.username_to_conn[&to_add.username],
            })
        } else {
            None
        };
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

    pub fn get_username(&self, conn_id: &UID) -> Option<String> {
        self.conn_to_user.get(&conn_id).map_or(None, |c| Some(c.username.clone()))
    }
}

#[derive(Debug)]
pub struct ConnectionManager {
    send_to_clients: UnboundedSender<ConnectionMessageWrap>,
    receive_server_messages: UnboundedReceiver<GameMessageWrap>,
    login_manager: LoginManager,
}
impl ConnectionManager {
    // TODONEXT: on init, creates a server. Then saves the returned channels.
    // then has methods, receive message and send message.
    // it should manage username:conn_uid and stuff here?
    pub async fn new() -> Self {
        let (send_to_clients, receive_server_messages) = create_server().await;
        ConnectionManager {
            send_to_clients,
            receive_server_messages,
            login_manager: LoginManager::new(),
        }
    }
    pub fn send_message(message: GameMessage, username: String) {

    }
    pub fn get_messages(&self) -> Vec<GameMessage> {
        // TODONEXT: Read all the current messages on the list.
        // if any are UserLogin, check if valid, if invalid respond with error?
        // if any DropConnection messages, handle here, just remove_connection from LoginManager
        vec![]
    }
}
