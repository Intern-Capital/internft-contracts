#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:internnft-staking-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw721(deps, env, info, msg),
    }
}

pub fn receive_cw721(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw721_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw721_msg.msg) {
        Ok(Cw721HookMsg::StakeGold {}) => stake_gold(deps, env, info.sender, cw721_msg),
        Ok(Cw721HookMsg::StakeExp {}) => stake_exp(deps, env, info.sender, cw721_msg),
        Err(_) => Err(ContractError::InvalidCw721ReceiveMsg {}),
    }
}

pub fn stake_exp(
    _deps: DepsMut,
    env: Env,
    _sender: Addr,
    _msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn stake_gold(
    _deps: DepsMut,
    _env: Env,
    _sender: Addr,
    _msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
