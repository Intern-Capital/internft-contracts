#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdResult, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::Cw721ReceiveMsg;
use internnft::nft::ExecuteMsg::UpdateTrait;
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
        Ok(Cw721HookMsg::Stake { staking_type }) => {
            stake(deps, env, info.sender, staking_type, cw721_msg)
        }
        Err(_) => Err(ContractError::InvalidCw721ReceiveMsg {}),
    }
}

pub fn stake(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    staking_type: String,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    if staking_type != "gold" && staking_type != "exp" {
        return Err(ContractError::InvalidStakingType {});
    }

    let config: Config = CONFIG.load(deps.storage)?;

    //if this returns an error, the token does not exist and we exit
    let token_info: InternTokenInfo =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.nft_contract_addr.to_string(),
            msg: to_binary(&InternNftInfo {
                token_id: msg.token_id.clone(),
            })?,
        }))?;

    let staking_info: StakingInfo = match STAKING_INFO.has(deps.storage, msg.token_id.clone()) {
        true => get_staking_info(&deps, msg.token_id.clone()).unwrap(),
        false => StakingInfo {
            staked: false,
            last_action_block_time: 0,
            current_stamina: token_info.extension.stamina,
            token_id: msg.token_id.clone(),
            owner: sender,
            staking_type: "".to_string(),
        },
    };

    let mut new_staking_info = staking_info.clone();

    new_staking_info.staked = true;
    new_staking_info.last_action_block_time = env.block.height;
    new_staking_info.staking_type = staking_type;

    //if the current stamina isn't the same as the max stamina in the NFT, then update the stamina
    if staking_info.current_stamina != token_info.extension.stamina {
        let stamina_to_add =
            (env.block.height - staking_info.last_action_block_time) * config.stamina_constant;
        new_staking_info.current_stamina =
            match token_info.extension.stamina > staking_info.current_stamina + stamina_to_add {
                true => staking_info.current_stamina + stamina_to_add,
                false => token_info.extension.stamina,
            };
    }

    STAKING_INFO.update(deps.storage, msg.token_id, |token| match token {
        None => Err(ContractError::NoStakedToken {}),
        Some(_) => Ok(new_staking_info),
    })?;

    //once stamina is updated, finish

    Ok(Response::new()) //TODO: add response
}

// all of the calculations for added exp and added gold are done upon unstaking
pub fn withdraw_nft(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    //check ownership and staking status of the NFT and return if it matches
    let config: Config = CONFIG.load(deps.storage)?;

    //if this returns an error, the token does not exist and we exit
    let token_info: InternTokenInfo =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.nft_contract_addr.to_string(),
            msg: to_binary(&InternNftInfo {
                token_id: token_id.clone(),
            })?,
        }))?;

    let staking_info: StakingInfo = match STAKING_INFO.has(deps.storage, token_id.clone()) {
        true => Ok(get_staking_info(&deps, token_id.clone()).unwrap()),
        false => Err(ContractError::NoStakedToken {}),
    }?;

    let mut new_staking_info: StakingInfo = staking_info.clone();
    let mut new_token_info: InternTokenInfo = token_info.clone();

    //update gold or experience
    //1. calculate stamina lost
    //1a. stamina_lost = blocks_elapsed * decay_rate (assuming linear decay)

    let stamina_lost = staking_info.current_stamina
        - (env.block.height - staking_info.last_action_block_time) * config.stamina_constant;

    //2. calculate the block times for which the rewards will be generated
    //2a. reward_blocks = [input_reward_block, output_reward_block]
    //  if updated_stamina = 0:
    //      output_reward_blocks = input_reward_block + input_stamina / decay_rate (this is assuming a linear decay rate)
    let input_reward_block = staking_info.last_action_block_time;
    let output_reward_block = match new_staking_info.current_stamina == 0 {
        true => input_reward_block + (staking_info.current_stamina / config.stamina_constant),
        false => env.block.height,
    };

    let mut added_exp = 0;
    let mut added_gold = 0;

    if staking_info.staking_type == "exp" {
        //3. calculate the exp to give
        //3a. exp = total_reward_blocks
        added_exp = (output_reward_block - input_reward_block) * config.exp_constant;
    } else if staking_info.staking_type == "gold" {
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

        let mut reward_block = 0;

        while reward_block < output_reward_block - input_reward_block {
            let wasm = WasmQuery::Smart {
                contract_addr: config.terrand_addr.to_string(),
                msg: to_binary(&GetRandomness {
                    round: current_round - reward_block,
                })?,
            };
            let res: GetRandomResponse = deps.querier.query(&wasm.into())?;
            for slice in res.randomness.as_slice() {
                added_gold += *slice as u64;
                reward_block += 1;
                if reward_block >= output_reward_block - input_reward_block {
                    break;
                }
            }
        }
    } else {
        return Err(ContractError::InvalidStakingType {})
    }
    //updating stamina, exp, gold at the end
    new_staking_info.current_stamina = match staking_info.current_stamina < stamina_lost {
        true => 0,
        false => staking_info.current_stamina - stamina_lost,
    };
    new_staking_info.staked = false;
    new_staking_info.last_action_block_time = env.block.height;

    STAKING_INFO.save(deps.storage, token_id.clone(), &new_staking_info)?;

    //updating the token information
    new_token_info.extension.experience = token_info.extension.experience + added_exp;
    new_token_info.extension.gold = token_info.extension.gold + added_gold;

    let message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract_addr.to_string(),
        msg: to_binary(&UpdateTrait {
            token_id,
            exp: new_token_info.extension.experience,
            gold: new_token_info.extension.gold,
            stamina: token_info.extension.stamina,
        })?,
        funds: vec![],
    });

    Ok(Response::new().add_message(message)) //TODO: add response
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
