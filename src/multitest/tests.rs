use cosmwasm_std::{Addr, Uint128, coins};
use cw_multi_test::App;

use crate::{msg::{TotalUserBidResp, HighestBidResp, IsClosedResp, WinnerResp}, state::BID_DENOM, error::ContractError};

use super::contract::BiddingContract;

#[test]
fn query_total_user_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("alex");
    let bid_amount1 = coins(100u128, BID_DENOM);
    let bid_amount2 = coins(200u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(300, BID_DENOM))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();

    contract.bid(&mut app, &sender, &bid_amount1).unwrap();
    contract.bid(&mut app, &sender, &bid_amount2).unwrap();

    let resp = contract.query_total_user_bid(&app, sender.to_string()).unwrap();
    assert_eq!(resp, TotalUserBidResp { amount: Uint128::from(270u128) });
}

#[test]
fn query_highest_bid_resp() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("alex");
    let sender2 = Addr::unchecked("anna");
    let bid_amount1 = coins(100u128, BID_DENOM);
    let bid_amount2 = coins(200u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router.bank
            .init_balance(storage, &sender1, coins(100, BID_DENOM))
            .unwrap();
        router.bank
            .init_balance(storage, &sender2, coins(200, BID_DENOM))
            .unwrap();
    });
    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();


    contract.bid(&mut app, &sender1, &bid_amount1).unwrap();
    contract.bid(&mut app, &sender2, &bid_amount2).unwrap();

    let resp = contract.query_highest_bid_resp(&app).unwrap();
    assert_eq!(resp, Some(HighestBidResp {address: sender2, amount: Uint128::from(180u128)}));
}

#[test]
fn query_is_closed() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("alex");
    let bid_amount1 = coins(100u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(100, BID_DENOM))
            .unwrap();
    });
    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();

    let resp = contract.query_is_closed(&app).unwrap();
    assert_eq!(resp, IsClosedResp {closed: false});

    contract.bid(&mut app, &sender1, &bid_amount1).unwrap();
    contract.close(&mut app, &owner).unwrap();

    let resp = contract.query_is_closed(&app).unwrap();
    assert_eq!(resp, IsClosedResp {closed: true});
}

#[test]
fn query_winner() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("alex");
    let sender2 = Addr::unchecked("anna");
    let bid_amount1 = coins(100u128, BID_DENOM);
    let bid_amount2 = coins(200u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router.bank
            .init_balance(storage, &sender1, coins(100, BID_DENOM))
            .unwrap();
        router.bank
            .init_balance(storage, &sender2, coins(200, BID_DENOM))
            .unwrap();
    });
    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();

    contract.bid(&mut app, &sender1, &bid_amount1).unwrap();
    assert_eq!(
        contract.query_highest_bid_resp(&app).unwrap(),
        Some(HighestBidResp {address: sender1, amount: Uint128::from(90u128)}),
    );
    assert_eq!(contract.query_winner(&app).unwrap(), None);


    contract.bid(&mut app, &sender2, &bid_amount2).unwrap();
    contract.close(&mut app, &owner).unwrap();
    let resp = contract.query_winner(&app).unwrap();
    assert_eq!(resp, Some(WinnerResp {address: sender2, amount: Uint128::from(180u128)}));
}

#[test]
fn close() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("alex");
    let sender2 = Addr::unchecked("anna");
    let bid_amount1 = coins(100u128, BID_DENOM);
    let bid_amount2 = coins(200u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router.bank
            .init_balance(storage, &sender1, coins(100, BID_DENOM))
            .unwrap();
        router.bank
            .init_balance(storage, &sender2, coins(200, BID_DENOM))
            .unwrap();
    });
    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();

    contract.bid(&mut app, &sender1, &bid_amount1).unwrap();
    contract.close(&mut app, &owner).unwrap();

    let err = contract.bid(&mut app, &sender2, &bid_amount2).unwrap_err();
    assert_eq!(
        err,
        ContractError::UnauthorizedWhileClosed {},
    );
}

#[test]
fn retract() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("alex");
    let sender2 = Addr::unchecked("anna");
    let bid_amount1 = coins(100u128, BID_DENOM);
    let bid_amount2 = coins(200u128, BID_DENOM);

    let mut app = App::new(|router, _api, storage| {
        router.bank
            .init_balance(storage, &sender1, coins(100, BID_DENOM))
            .unwrap();
        router.bank
            .init_balance(storage, &sender2, coins(200, BID_DENOM))
            .unwrap();
    });
    let code_id = BiddingContract::store_code(&mut app);
    let contract = BiddingContract::get_default_contract(&mut app, code_id, &owner).unwrap();

    contract.bid(&mut app, &sender1, &bid_amount1).unwrap();
    contract.bid(&mut app, &sender2, &bid_amount2).unwrap();

    let err = contract.retract(&mut app, &sender1, None).unwrap_err();
    assert_eq!(
        err,
        ContractError::UnauthorizedWhileOpen {},
    );

    contract.close(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(&sender1).unwrap(),
        &[],
    );
    contract.retract(&mut app, &sender1, None).unwrap();
    assert_eq!(
        app.wrap().query_all_balances(&sender1).unwrap(),
        coins(90, BID_DENOM),
    );

    let err = contract.retract(&mut app, &sender2, None).unwrap_err();
    assert_eq!(err, ContractError::InvalidRetractZeroAmount {});

}
