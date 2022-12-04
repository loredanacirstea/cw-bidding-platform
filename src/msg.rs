use cosmwasm_std::{Addr, Decimal};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub commodity_uri: String,
    pub owner: Option<String>,
    pub bid_comission: Decimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TotalUserBidResp)]
    TotalUserBid {
        // #[serde(default)]
        address: String,
    },

    #[returns(HighestBidResp)]
    HighestBid {},

    #[returns(IsClosedResp)]
    IsClosed {},

    #[returns(Option<WinnerResp>)]
    Winner {},
}

#[cw_serde]
pub enum ExecMsg {
    Bid {},
    Close {},
    Retract {
        #[serde(default)]
        receiver: String,
    },
}

#[cw_serde]
pub struct TotalUserBidResp {
    pub amount: u64,
}

#[cw_serde]
pub struct HighestBidResp {
    pub address: Addr,
    pub amount: u64,
}

#[cw_serde]
pub struct IsClosedResp {
    pub closed: bool,
}

#[cw_serde]
pub struct WinnerResp {
    pub address: Addr,
    pub amount: u64,
}
