use crate::ContractError;
use cosmwasm_std::DepsMut;
use cw_storage_plus::{Item, Map};
use internnft::staking::{Config, StakingInfo};

pub const CONFIG: Item<Config> = Item::new("config");

// map containing the information for all of the tokens that have underwent staking
pub const STAKING_INFO: Map<String, StakingInfo> = Map::new("stakers");

pub fn get_staking_info(deps: &DepsMut, token_id: String) -> Result<StakingInfo, ContractError> {
    match STAKING_INFO.load(deps.storage, token_id) {
        Ok(staking_info) => Ok(staking_info),
        Err(_) => Err(ContractError::NoStakedToken {}),
    }
}
