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
    pub fn highest_bid(deps: Deps) -> StdResult<Option<HighestBidResp>> {
        let winner = WINNER.may_load(deps.storage)?;
        match winner {
            Some(i) => Ok(Some(HighestBidResp { address: i.address, amount: i.amount })),
            None => Ok(None),
        }
    }

    pub fn is_closed(deps: Deps) -> StdResult<IsClosedResp> {
        let closed = STATE.load(deps.storage)?.is_closed;
        Ok(IsClosedResp { closed })
    }

    pub fn winner(deps: Deps) -> StdResult<Option<WinnerResp>> {
        let closed = STATE.load(deps.storage)?.is_closed;
        if closed == false {
            return Ok(None);
        }

        let winner = WINNER.may_load(deps.storage)?;
        match winner {
            Some(i) => Ok(Some(WinnerResp { address: i.address, amount: i.amount })),
            None => Ok(None),
        }
    }
}

pub mod exec {
    use std::ops::{Sub, Add};
    use std::str::FromStr;

    use cosmwasm_std::{
        DepsMut, MessageInfo, Response, BankMsg, coins, Uint128, Decimal,
    };

    use crate::error::ContractError;
    use crate::state::{STATE, WINNER, BIDS, BID_DENOM, Winner};

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.is_closed {
            return Err(ContractError::UnauthorizedWhileClosed {});
        }

        let current_winner = WINNER.may_load(deps.storage)?;
        let winner_amount = match current_winner {
            Some(i) => i.amount,
            None => Uint128::zero(),
        };
        let mut user_bid = BIDS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default();

        let amount = info.funds.iter().find(|coin| coin.denom == BID_DENOM.to_string());

        let coin_bid = match amount {
            Some(i) => i,
            None => return Err(ContractError::InvalidBidZeroAmount {}),
        };

        let commission = Decimal::from_atomics(coin_bid.amount, 0)?
            .checked_mul(state.bid_comission)?
            .ceil();

        let amount_commission = Uint128::from_str(&commission.to_string())?;
        let amount_bid = coin_bid.amount.sub(amount_commission);
        user_bid = user_bid.checked_add(amount_bid)?;

        if !winner_amount.lt(&user_bid) {
            let required_amount = winner_amount
                .sub(user_bid)
                .add(amount_bid)
                .add(Uint128::one());
            return Err(ContractError::InvalidBidAmount {amount: amount_bid, required_amount})
        }

        BIDS.save(deps.storage, &info.sender, &user_bid)?;

        let winner = Winner{amount: user_bid, address: info.sender.clone()};
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


        let winner = WINNER.may_load(deps.storage)?;
        match winner {
            Some(i) => {
                // Store 0 for winner's bid
                BIDS.save(deps.storage, &i.address, &Uint128::zero())?;

                // Send winner's amount to owner
                let bank_msg = BankMsg::Send {
                    to_address: state.owner.to_string(),
                    amount: coins(i.amount.u128(), BID_DENOM),
                };
                resp = resp
                    .add_message(bank_msg)
                    .add_attribute("action", "close")
                    .add_attribute("winner", i.address.as_str())
                    .add_attribute("amount", i.amount.to_string());
            },
            None => {
                resp = resp
                    .add_attribute("action", "close")
            },
        }

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

        if amount.is_zero() {
            return Err(ContractError::InvalidRetractZeroAmount {});
        }

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
