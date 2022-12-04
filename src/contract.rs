use cosmwasm_std::{DepsMut, Response, StdResult, Decimal, MessageInfo};
use cw2::{set_contract_version};
use crate::state::{State, STATE};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    commodity_uri: String,
    bid_comission: Decimal,
    owner: Option<String>,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let contract_owner = match owner {
        Some(i) => deps.api.addr_validate(&i)?,
        None => info.sender,
    };

    STATE.save(
        deps.storage,
        &State {
            commodity_uri,
            owner: contract_owner,
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
    use std::ops::{Sub, Add};

    use cosmwasm_std::{
        DepsMut, MessageInfo, Response, BankMsg, coins, Uint128, Decimal,
    };

    use crate::error::ContractError;
    use crate::state::{STATE, WINNER, BIDS, BID_DENOM};

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.is_closed {
            return Err(ContractError::UnauthorizedWhileClosed {});
        }

        let mut winner = WINNER.load(deps.storage)?;
        let mut user_bid = BIDS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default();

        let amount = info.funds.iter().find(|coin| coin.denom == BID_DENOM.to_string());

        let coin_bid = match amount {
            Some(i) => i,
            None => return Err(ContractError::InvalidBidZeroAmount {}),
        };

        let amount_commission = Decimal::from_atomics(coin_bid.amount, 0)?
            .checked_div(state.bid_comission)?
            .ceil()
            .atomics();

        let amount_bid = coin_bid.amount.sub(amount_commission);
        user_bid = user_bid.checked_add(amount_bid)?;

        if !winner.amount.lt(&user_bid) {
            let required_amount = winner.amount
                .sub(user_bid)
                .add(amount_bid)
                .add(Uint128::one());
            return Err(ContractError::InvalidBidAmount {amount: amount_bid, required_amount})
        }

        BIDS.save(deps.storage, &info.sender, &user_bid)?;

        winner.amount = user_bid;
        winner.address = info.sender.clone();
        WINNER.save(deps.storage, &winner)?;

        // Send winner's amount to owner
        let bank_msg = BankMsg::Send {
            to_address: state.owner.to_string(),
            amount: coins(amount_commission.u128(), BID_DENOM),
        };

        resp = resp
            .add_message(bank_msg)
            .add_attribute("action", "bid")
            .add_attribute("bidder", info.sender.as_str())
            .add_attribute("amount", info.funds[0].to_string());

        Ok(resp)
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.is_closed {
            return Err(ContractError::UnauthorizedWhileClosed {});
        }

        if state.owner != info.sender {
            return Err(ContractError::Unauthorized {
                owner: state.owner.to_string(),
            });
        }

        state.is_closed = true;
        STATE.save(deps.storage, &state)?;

        let winner = WINNER.load(deps.storage)?;

        // Store 0 for winner's bid
        BIDS.save(deps.storage, &winner.address, &Uint128::zero())?;

        // Send winner's amount to owner
        let bank_msg = BankMsg::Send {
            to_address: state.owner.to_string(),
            amount: coins(winner.amount.u128(), BID_DENOM),
        };

        resp = resp
            .add_message(bank_msg)
            .add_attribute("action", "close")
            .add_attribute("winner", winner.address.as_str())
            .add_attribute("amount", winner.amount.to_string());

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, info: MessageInfo, receiver: Option<String>) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.is_closed == false {
            return Err(ContractError::UnauthorizedWhileOpen {});
        }

        let funds_receiver = match receiver {
            Some(i) => deps.api.addr_validate(&i)?,
            None => info.sender.clone(),
        };

        let amount = BIDS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default();

        // Store 0 for bidder
        BIDS.save(deps.storage, &info.sender, &Uint128::zero())?;

        // Send funds back to bidder
        let bank_msg = BankMsg::Send {
            to_address: funds_receiver.to_string(),
            amount: coins(amount.u128(), BID_DENOM),
        };

        resp = resp
            .add_message(bank_msg)
            .add_attribute("action", "retract")
            .add_attribute("bidder", info.sender.as_str())
            .add_attribute("receiver", funds_receiver.as_str())
            .add_attribute("amount", amount.to_string());

        Ok(resp)
    }
}
