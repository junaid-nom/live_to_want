use crate::{CreatureCommandUser, GameState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameMessage {
    StringMsg(String),
    GameStateMsg(GameState),
    CreatureCommandMsg(CreatureCommandUser),
}
