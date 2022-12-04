use cosmwasm_std::{Addr, DepsMut, Response, StdResult, Decimal};
use cw2::{set_contract_version};
// use cw_storage_plus::Item;
// use serde::{Deserialize, Serialize};

// use crate::error::ContractError;
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    commodity_uri: String,
    bid_comission: Decimal,
    owner: Addr,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        &State {
            commodity_uri,
            owner,
            bid_comission,
            is_closed: false,
        },
    )?;

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::{TotalUserBidResp, HighestBidResp, IsClosedResp, WinnerResp};
    use crate::state::{STATE, BIDS, WINNER};

    pub fn total_user_bid(deps: Deps, address: String) -> StdResult<TotalUserBidResp> {
        let address = deps.api.addr_validate(&address)?;
        let amount = BIDS
            .may_load(deps.storage, &address)?
            .unwrap_or_default();
        Ok(TotalUserBidResp { amount })
    }

    // we show this even if bid is closed
    pub fn highest_bid(deps: Deps) -> StdResult<HighestBidResp> {
        let winner = WINNER.load(deps.storage)?;
        Ok(HighestBidResp { address: winner.address, amount: winner.amount })
    }

    pub fn is_closed(deps: Deps) -> StdResult<IsClosedResp> {
        let closed = STATE.load(deps.storage)?.is_closed;
        Ok(IsClosedResp { closed })
    }

    pub fn winner(deps: Deps) -> StdResult<Option<WinnerResp>> {
        let closed = STATE.load(deps.storage)?.is_closed;
        if closed == true {
            return Ok(None);
        }

        let winner = WINNER.load(deps.storage)?;
        Ok(Some(WinnerResp { address: winner.address, amount: winner.amount }))
    }
}

pub mod exec {
    // use cosmwasm_std::{
    //     to_binary, DepsMut, Env, MessageInfo, Response, StdResult,
    // };

    // use crate::error::ContractError;
    // use crate::msg::ExecMsg;
    // use crate::state::{STATE};

    // pub fn bid(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // }

    // pub fn close(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // }

    // pub fn retract(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // }
}
