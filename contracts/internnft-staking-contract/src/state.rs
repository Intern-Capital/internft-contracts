use cosmwasm_std::{Addr, DepsMut, StdResult};
use cw_storage_plus::Map;
use crate::ContractError;

pub struct StakingInfo {
    pub input_block_height: u64,
    pub initial_stamina: u8,
    pub token_id: String,
    pub owner: Addr,
    pub staking_type: String,
}

pub const STAKERS: Map<Addr, Vec<StakingInfo>> = Map::new("stakers");

pub fn get_staking_info(deps: &DepsMut, address: Addr) -> Result<Vec<StakingInfo>, ContractError> {
    match STAKERS.load(deps.storage, address) {
        Ok(staking_info) => Ok(staking_info),
        Err(_) => Err(ContractError::NoStakedToken {}),
    }
}