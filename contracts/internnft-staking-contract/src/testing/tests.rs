use cosmwasm_std::{Addr, Response, to_binary};
use cosmwasm_std::testing::{mock_env, mock_info};
use cw721::Cw721ReceiveMsg;
use internnft::staking::{Config, Cw721HookMsg, InstantiateMsg};
use internnft::staking::Cw721HookMsg::Stake;
use crate::contract::{instantiate, query_config, stake};
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

    let instantiate_res = instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg.clone()).unwrap();

    let hook_msg = Cw721HookMsg::Stake { staking_type: "gold".to_string() };

    let receive_msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: "0".to_string(),
        msg: to_binary(&hook_msg).unwrap(),
    };

    let staking_res = stake(deps.as_mut(), env, info.sender, "gold".to_string(), receive_msg).unwrap();

    let test_staking_res = Response::new()
        .add_attribute("action", "stake")
        .add_attribute("token_id", "0".to_string())
        .add_attribute("staking_type", "gold".to_string());

    assert_eq!(staking_res, test_staking_res);
}

#[test]
fn test_exp_staking() {
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