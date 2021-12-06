use cosmwasm_std::{Addr, Order, StdError, StdResult, Storage};
use cw_storage_plus::Map;

pub struct StakingInfo {
    pub input_block_height: u64,
    pub initial_stamina: u8,
    pub token_id: String,
    pub owner: Addr,
    pub staking_type: String,
}

pub const STAKERS: Map<Addr, StakingInfo> = Map::new("stakers");

pub fn all_staking_addresses(storage: &dyn Storage) -> StdResult<Vec<String>> {
    STAKERS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| Addr::unchecked(String::from_utf8(k)).map_err(|_| StdError::invalid_utf8("parsing address")))
        .collect()
}

