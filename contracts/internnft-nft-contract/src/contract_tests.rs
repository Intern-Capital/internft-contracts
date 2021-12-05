#![cfg(test)]
use std::str;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{BankMsg, Binary, Coin, DepsMut};
use internnft::nft::{Config, InstantiateMsg};

use crate::contract::instantiate;
use crate::error::ContractError;
use crate::execute as ExecHandler;
use crate::query as QueryHandler;

const OWNER: &str = "owner";
const NONOWNER: &str = "nonowner";

fn mock_config() -> Config {
    Config {
        mint_fee: Coin::new(0, "uluna"),
        token_supply: 10000,
        wallet_limit: 5,
    }
}

fn setup_contract(
    deps: DepsMut,
    mint_fee: Option<Coin>,
    token_supply: Option<u64>,
    wallet_limit: Option<u32>,
) {
    let mut msg = InstantiateMsg {
        config: mock_config(),
    };
    if let Some(mint_fee) = mint_fee {
        msg.config.mint_fee = mint_fee;
    }
    if let Some(token_supply) = token_supply {
        msg.config.token_supply = token_supply;
    }
    if let Some(wallet_limit) = wallet_limit {
        msg.config.wallet_limit = wallet_limit;
    }
    let info = mock_info(OWNER, &[]);
    let res = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

fn as_json(binary: &Binary) -> serde_json::Value {
    let b64_binary = binary.to_base64();
    let decoded_bytes = base64::decode(&b64_binary).unwrap();
    let decoded_str = str::from_utf8(&decoded_bytes).unwrap();
    serde_json::from_str(decoded_str).unwrap()
}

#[test]
fn update_and_query_config() {
    let initial_config = mock_config();

    let mut deps = mock_dependencies(&[]);
    setup_contract(
        deps.as_mut(),
        Some(initial_config.mint_fee.clone()),
        Some(initial_config.token_supply),
        Some(initial_config.wallet_limit),
    );

    // query initial config
    let res = QueryHandler::query_config(deps.as_ref()).unwrap();
    assert_eq!(res, initial_config);

    // change the config
    let mut new_config = initial_config.clone();
    new_config.mint_fee = Coin::new(10000, "uluna");

    // nonowner can't update config
    let err = ExecHandler::execute_update_config(
        deps.as_mut(),
        mock_info(NONOWNER, &[]),
        new_config.clone(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // check config was unchanged
    let res = QueryHandler::query_config(deps.as_ref()).unwrap();
    assert_eq!(res, initial_config);

    // owner can update config
    let _ = ExecHandler::execute_update_config(
        deps.as_mut(),
        mock_info(OWNER, &[]),
        new_config.clone(),
    )
    .unwrap();

    // check config was updated
    let res = QueryHandler::query_config(deps.as_ref()).unwrap();
    assert_eq!(res, new_config);
}

#[test]
fn withdraw() {
    let balance = vec![Coin::new(10000, "uluna")];
    let mut deps = mock_dependencies(&balance);
    setup_contract(deps.as_mut(), None, None, None);

    // non-owner can't withdraw
    let err = ExecHandler::execute_withdraw(
        deps.as_mut(),
        mock_env(),
        mock_info(NONOWNER, &[]),
        vec![Coin::new(100, "uluna")],
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // owner can withdraw
    let res = ExecHandler::execute_withdraw(
        deps.as_mut(),
        mock_env(),
        mock_info(OWNER, &[]),
        vec![Coin::new(100, "uluna")],
    )
    .unwrap();
    assert_eq!(
        res.messages[0].msg,
        BankMsg::Send {
            amount: vec![Coin::new(100, "uluna")],
            to_address: mock_info(OWNER, &[]).sender.to_string()
        }
        .into()
    )
}
