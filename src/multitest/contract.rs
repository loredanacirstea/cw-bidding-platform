use cosmwasm_std::{Addr, Coin, StdResult, Uint128, Decimal};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg, QueryMsg, TotalUserBidResp, HighestBidResp, IsClosedResp, WinnerResp};
use crate::{execute, instantiate, query};

pub struct BiddingContract(Addr);

impl BiddingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    pub fn get_default_contract(app: &mut App, code_id: u64, owner: &Addr) -> StdResult<Self> {
        BiddingContract::instantiate(
            app,
            code_id,
            owner,
            "Bidding contract",
            None,
            "someuri".to_string(),
            Decimal::from_atomics(Uint128::one(), 1).unwrap(),
            None,
        )
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        admin: impl Into<Option<&'a Addr>>,
        commodity_uri: String,
        bid_comission: Decimal,
        owner: Option<String>,
    ) -> StdResult<Self> {
        let admin = admin.into();

        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                commodity_uri,
                owner,
                bid_comission,
            },
            &[],
            label,
            admin.map(Addr::to_string),
        )
        .map(BiddingContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn bid(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Bid {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn close(
        &self,
        app: &mut App,
        sender: &Addr,
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Close {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn retract(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: Option<String>,
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Retract { receiver }, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn query_total_user_bid(&self, app: &App, address: String) -> StdResult<TotalUserBidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::TotalUserBid { address })
    }

    #[track_caller]
    pub fn query_highest_bid_resp(&self, app: &App) -> StdResult<Option<HighestBidResp>> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }

    #[track_caller]
    pub fn query_is_closed(&self, app: &App) -> StdResult<IsClosedResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::IsClosed {})
    }

    #[track_caller]
    pub fn query_winner(&self, app: &App) -> StdResult<Option<WinnerResp>> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Winner {})
    }
}

impl From<BiddingContract> for Addr {
    fn from(contract: BiddingContract) -> Self {
        contract.0
    }
}
