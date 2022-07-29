use std::fmt::Formatter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub players: [Addr; 2],
    pub board: [[GridCell; 3]; 3],
    pub next_turn: Turn,
    pub winner: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Turn {
    Player0,
    Player1,
    Ended
}

impl ::std::fmt::Display for Turn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item = match self {
            Turn::Player0 => "X",
            Turn::Player1 => "O",
            _ => "invalid"
        };
        write!(f, "{}", item)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Copy)]
pub enum GridCell {
    Empty,
    X,
    O,
}

pub const STATE: Item<State> = Item::new("state");
