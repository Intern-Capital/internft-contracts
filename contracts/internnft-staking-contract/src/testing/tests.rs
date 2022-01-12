use cosmwasm_std::{Addr, CosmosMsg, Response, Timestamp, to_binary, WasmMsg};
use cosmwasm_std::testing::{mock_env, mock_info};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use internnft::nft::ExecuteMsg::UpdateTrait;
use internnft::staking::{Config, Cw721HookMsg, InstantiateMsg, StakingInfo};


use crate::contract::{instantiate, query_config, query_staking_info, stake, withdraw_nft};
use crate::ContractError;
use crate::testing::mock_querier::mock_dependencies;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(0, res.messages.len());
    let test_response: Response = Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner)
        .add_attribute("nft_contract_address", msg.nft_contract_addr)
        .add_attribute("terrand_addr", msg.terrand_addr)
        .add_attribute("stamina_constant", msg.stamina_constant.to_string())
        .add_attribute("exp_constant", msg.exp_constant.to_string());
    assert_eq!(res, test_response);
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let query_res = query_config(deps.as_ref()).unwrap();

    let test_config: Config = Config {
        nft_contract_addr: msg.nft_contract_addr,
        terrand_addr: msg.terrand_addr,
        owner: msg.owner,
        stamina_constant: 1,
        exp_constant: 1
    };

    assert_eq!(to_binary(&test_config).unwrap(), query_res);
}

#[test]
fn test_gold_staking() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "gold".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "gold".to_string(), receive_msg).unwrap();

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "gold".to_string());

    assert_eq!(staking_res, test_staking_res);

    let query_staking_res = query_staking_info(deps.as_ref(), "0".to_string()).unwrap();

    let test_staking_res = to_binary(&StakingInfo {
        staked: true,
        last_action_block_time: env.block.height,
        current_stamina: 100,
        token_id: "0".to_string(),
        owner: info.sender,
        staking_type: "gold".to_string()
    }).unwrap();

    assert_eq!(query_staking_res, test_staking_res);
}

#[test]
fn test_exp_staking() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "exp".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "exp".to_string(), receive_msg).unwrap();

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "exp".to_string());

    assert_eq!(staking_res, test_staking_res);

    let query_staking_res = query_staking_info(deps.as_ref(), "0".to_string()).unwrap();

    let test_staking_res = to_binary(&StakingInfo {
        staked: true,
        last_action_block_time: env.block.height,
        current_stamina: 100,
        token_id: "0".to_string(),
        owner: info.sender,
        staking_type: "exp".to_string()
    }).unwrap();

    assert_eq!(query_staking_res, test_staking_res);
}

#[test]
fn test_staking_token_twice() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "exp".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "exp".to_string(), receive_msg.clone()).unwrap();

    let staking_twice_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "exp".to_string(), receive_msg);

    match staking_twice_res {
        Err(ContractError::TokenAlreadyStaked {}) => (),
        _ => panic!("Must return token already staked error"),
    }

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "exp".to_string());

    assert_eq!(staking_res, test_staking_res);

    let query_staking_res = query_staking_info(deps.as_ref(), "0".to_string()).unwrap();

    let test_staking_res = to_binary(&StakingInfo {
        staked: true,
        last_action_block_time: env.block.height,
        current_stamina: 100,
        token_id: "0".to_string(),
        owner: info.sender,
        staking_type: "exp".to_string()
    }).unwrap();

    assert_eq!(query_staking_res, test_staking_res);
}

#[test]
fn test_staking_unowned_token() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0001", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "exp".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env, info.sender, "exp".to_string(), receive_msg);

    match staking_res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn test_stake_unstake_gold_stamina_not_depleted() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();
    let gold_rewards: Vec<u64> = vec![1, 0, 2, 3, 2, 1, 2, 0, 3, 1, 2, 1, 3, 2, 2, 0, 0, 0, 2, 0, 3, 0, 2, 2, 1, 0, 0, 3, 1, 0, 0, 3, 3, 2, 0, 1, 0, 1, 3, 3, 3, 3, 1, 1];
    env.block.time = Timestamp::from_seconds(1595431050+1000000);
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "gold".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "gold".to_string(), receive_msg).unwrap();

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "gold".to_string());

    assert_eq!(staking_res, test_staking_res);

    let query_staking_res = query_staking_info(deps.as_ref(), "0".to_string()).unwrap();

    let test_staking_res = to_binary(&StakingInfo {
        staked: true,
        last_action_block_time: env.block.height,
        current_stamina: 100,
        token_id: "0".to_string(),
        owner: info.sender.clone(),
        staking_type: "gold".to_string()
    }).unwrap();

    assert_eq!(query_staking_res, test_staking_res);

    let staked_blocks = 10;
    env.block.height += staked_blocks;

    let unstake_res = withdraw_nft(deps.as_mut(), env, info, "0".to_string()).unwrap();

    let mut added_gold = 0;
    for i in 0..staked_blocks as usize {
        added_gold+=gold_rewards[i];
    }

    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "internnft0000".to_string(),
            msg: to_binary(&UpdateTrait {
                token_id: "0".to_string(),
                exp: 0,
                gold: added_gold,
                stamina: 100,
            }).unwrap(),
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "internnft0000".to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: "addr0000".to_string(),
                token_id: "0".to_string(),
            }).unwrap(),
            funds: vec![]
        })
    ];

    let unstake_test_res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "unstake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "gold".to_string())
        .add_attribute("gold_added", added_gold.to_string())
        .add_attribute("exp_added", 0.to_string())
        .add_attribute("stamina_lost", staked_blocks.to_string())
        .add_attribute("new_stamina", 90.to_string());

    assert_eq!(unstake_res, unstake_test_res)
}

#[test]
fn test_stake_unstake_exp_stamina_not_depleted() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1595431050+1000000);
    let instantiate_msg = InstantiateMsg {
        owner: Addr::unchecked("owner0000"),
        nft_contract_addr: Addr::unchecked("internnft0000"),
        terrand_addr: Addr::unchecked("terrand0000"),
        stamina_constant: 1,
        exp_constant: 1
    };

    let info = mock_info("addr0000", &[]);

    let _instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "gold".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env.clone(), info.sender.clone(), "exp".to_string(), receive_msg).unwrap();

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "exp".to_string());

    assert_eq!(staking_res, test_staking_res);

    let query_staking_res = query_staking_info(deps.as_ref(), "0".to_string()).unwrap();

    let test_staking_res = to_binary(&StakingInfo {
        staked: true,
        last_action_block_time: env.block.height,
        current_stamina: 100,
        token_id: "0".to_string(),
        owner: info.sender.clone(),
        staking_type: "exp".to_string()
    }).unwrap();

    assert_eq!(query_staking_res, test_staking_res);

    let staked_blocks = 10;
    env.block.height += staked_blocks;

    let unstake_res = withdraw_nft(deps.as_mut(), env, info, "0".to_string()).unwrap();

    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "internnft0000".to_string(),
            msg: to_binary(&UpdateTrait {
                token_id: "0".to_string(),
                exp: staked_blocks,
                gold: 0,
                stamina: 100,
            }).unwrap(),
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "internnft0000".to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: "addr0000".to_string(),
                token_id: "0".to_string(),
            }).unwrap(),
            funds: vec![]
        })
    ];

    let unstake_test_res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "unstake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "exp".to_string())
        .add_attribute("gold_added", 0.to_string())
        .add_attribute("exp_added", staked_blocks.to_string())
        .add_attribute("stamina_lost", staked_blocks.to_string())
        .add_attribute("new_stamina", 90.to_string());

    assert_eq!(unstake_res, unstake_test_res)
}