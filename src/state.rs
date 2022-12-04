use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub commodity_uri: String,
    pub owner: Addr,
    pub bid_comission: Decimal,
    pub is_closed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Winner {
    pub address: Addr,
    pub amount: u64,
}

// pub const BID_DENOM: &str = "ATOM";
pub const STATE: Item<State> = Item::new("state");
pub const BIDS: Map<&Addr, u64> = Map::new("bids");
pub const WINNER: Item<Winner> = Item::new("winner");
