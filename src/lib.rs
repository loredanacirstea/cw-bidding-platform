#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use error::ContractError;
use msg::{InstantiateMsg};

mod contract;
pub mod error;
pub mod msg;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // let owner = deps.api.addr_validate(&Some(msg.owner))?;
    let owner = match msg.owner {
        Some(i) => deps.api.addr_validate(&i)?,
        None => info.sender,
    };

    contract::instantiate(deps, msg.commodity_uri, msg.bid_comission, owner)
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn execute(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: msg::ExecMsg,
// ) -> Result<Response, ContractError> {
//     use contract::exec;
//     use msg::ExecMsg::*;

//     match msg {
//         // Bid {} => exec::bid(deps, env, info).map_err(ContractError::Std),
//         // Close {} => exec::close(deps, env, info),
//         // Retract { receiver } => exec::retract(deps, env, info, receiver),
//     }
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;

    match msg {
        TotalUserBid { address } => to_binary(&query::total_user_bid(deps, address)?),
        HighestBid {} => to_binary(&query::highest_bid(deps)?),
        IsClosed {} => to_binary(&query::is_closed(deps)?),
        Winner {} => to_binary(&query::winner(deps)?),
    }
}
