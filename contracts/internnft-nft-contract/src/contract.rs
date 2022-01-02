use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use internnft::nft::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::execute as ExecHandler;
use crate::query as QueryHandler;

const CONTRACT_NAME: &str = "crates.io:internnft-nft-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ExecHandler::instantiate(deps, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => ExecHandler::execute_mint(deps, env, info),
        ExecuteMsg::UpdateConfig { config } => {
            ExecHandler::execute_update_config(deps, info, config)
        }
        ExecuteMsg::Withdraw { amount } => ExecHandler::execute_withdraw(deps, env, info, amount),
        ExecuteMsg::UpdateTrait { token_id, exp, gold, stamina } => ExecHandler::execute_update_traits(deps, env, info, token_id, exp, gold, stamina),
        _ => ExecHandler::cw721_base_execute(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryHandler::query_config(deps)?),
        QueryMsg::InternNftInfo { token_id } => {
            to_binary(&QueryHandler::query_intern_nft_info(deps, token_id)?)
        }
        QueryMsg::InternTokens {
            owner,
            start_after,
            limit,
        } => to_binary(&QueryHandler::query_intern_tokens(
            deps,
            owner,
            start_after,
            limit,
        )?),
        QueryMsg::AllInternTokens { start_after, limit } => to_binary(
            &QueryHandler::query_all_intern_tokens(deps, start_after, limit)?,
        ),
        QueryMsg::NumTokensForOwner { owner } => {
            to_binary(&QueryHandler::query_num_tokens_for_owner(deps, owner)?)
        }
        _ => QueryHandler::cw721_base_query(deps, env, msg),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err(
            "can't migrate to contract with different name",
        ));
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ExecHandler::migrate(deps, env, msg)
}
