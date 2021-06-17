use crate::{CreatureCommandUser, GameState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "msg_type")]
pub enum GameMessage {
    GameStateMsg(GameState),
    CreatureCommandMsg(CreatureCommandUser),
    StringMsg(String),
}
