#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmQuery,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;
use internnft::nft::InternTokenInfo;
use internnft::nft::QueryMsg::InternNftInfo;
use internnft::staking::ContractQuery::GetRandomness;
use internnft::staking::{
    Config, Cw721HookMsg, ExecuteMsg, GetRandomResponse, InstantiateMsg, QueryMsg, StakingInfo,
};

use crate::error::ContractError;
use crate::state::{get_staking_info, CONFIG, STAKING_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:internnft-staking-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
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
        ExecuteMsg::WithdrawNft { nft_id } => withdraw_nft(deps, env, info, nft_id),
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
    _env: Env,
    _sender: Addr,
    _msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn stake_gold(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    //if this returns an error, the token does not exist and we exit
    let token_info: InternTokenInfo =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.nft_contract_addr.to_string(),
            msg: to_binary(&InternNftInfo {
                token_id: msg.token_id.clone(),
            })?,
        }))?;

    let mut staking_info: StakingInfo = match STAKING_INFO.has(deps.storage, msg.token_id.clone()) {
        true => get_staking_info(&deps, msg.token_id).unwrap(),
        false => StakingInfo {
            staked: false,
            last_action_block_time: 0,
            current_stamina: token_info.extension.stamina,
            token_id: msg.token_id,
            owner: sender,
            staking_type: "".to_string(),
        },
    };

    staking_info.staked = true;
    staking_info.last_action_block_time = env.block.height;
    staking_info.staking_type = "gold".to_string();

    //if the current stamina isn't the same as the max stamina in the NFT, then update the stamina
    if staking_info.current_stamina != token_info.extension.stamina {
        //update stamina
    }

    //once stamina is updated, finish

    Ok(Response::new())
}

// all of the calculations for added exp and added gold are done upon unstaking
pub fn withdraw_nft(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    nft_id: String,
) -> Result<Response, ContractError> {
    //check ownership and staking status of the NFT and return if it matches
    let config: Config = CONFIG.load(deps.storage)?;

    //if this returns an error, the token does not exist and we exit
    let token_info: InternTokenInfo =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.nft_contract_addr.to_string(),
            msg: to_binary(&InternNftInfo {
                token_id: nft_id.clone(),
            })?,
        }))?;

    let staking_info: StakingInfo = match STAKING_INFO.has(deps.storage, nft_id.clone()) {
        true => Ok(get_staking_info(&deps, nft_id).unwrap()),
        false => Err(ContractError::NoStakedToken {}),
    }?;

    let mut new_staking_info: StakingInfo = staking_info.clone();
    let mut new_token_info: InternTokenInfo = token_info.clone();

    //update gold or experience
    //1. calculate stamina lost
    //1a. stamina_lost = blocks_elapsed * decay_rate (assuming linear decay)

    let stamina_lost = staking_info.current_stamina
        - (env.block.height - staking_info.last_action_block_time) * config.stamina_constant;
    new_staking_info.current_stamina = match staking_info.current_stamina < stamina_lost {
        true => 0,
        false => staking_info.current_stamina - stamina_lost,
    };

    //2. calculate the block times for which the rewards will be generated
    //2a. reward_blocks = [input_reward_block, output_reward_block]
    //  if updated_stamina = 0:
    //      output_reward_blocks = input_reward_block + input_stamina / decay_rate (this is assuming a linear decay rate)
    let input_reward_block = staking_info.last_action_block_time;
    let output_reward_block = match new_staking_info.current_stamina == 0 {
        true => input_reward_block + (staking_info.current_stamina / config.stamina_constant),
        false => env.block.height,
    };

    //3. calculate the exp to give
    //3a. exp = total_reward_blocks
    let added_exp = output_reward_block - input_reward_block;
    new_token_info.extension.experience = token_info.extension.experience + added_exp;

    //4. calculate the gold to give:
    //4a. gold =
    const GENESIS_TIME: u64 = 1595431050;
    const PERIOD: u64 = 30;

    let timestamp_now = env.block.time.seconds();

    // Get the current block time from genesis time
    let from_genesis = timestamp_now - GENESIS_TIME;

    // Get the current round
    let current_round = from_genesis / PERIOD;
    // Get the next round
    let _next_round = current_round + 1;

    let mut added_gold = 0;
    for i in 0..(output_reward_block - input_reward_block) {
        let wasm = WasmQuery::Smart {
            contract_addr: config.terrand_addr.to_string(),
            msg: to_binary(&GetRandomness {
                round: current_round - i,
            })?,
        };
        let res: GetRandomResponse = deps.querier.query(&wasm.into())?;

        //making this easy:
        //taking randomness from terrand
        //slicing it up
        //taking the first 8-bit value
        //modding it by 4
        //adding that onto added_gold
        //for a total of block_reward times

        //if this happens to be too much, we can change it
        added_gold += (res.randomness.as_slice()[0] % 4) as u64;
    }

    new_token_info.extension.gold = token_info.extension.gold + added_gold;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
