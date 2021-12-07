use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub struct StakingInfo {
    pub input_block_height: u64,
    pub initial_stamina: u8,
    pub token_id: String,
    pub owner: Addr,
    pub staking_type: String,
}

pub const STAKERS: Map<Addr, StakingInfo> = Map::new("stakers");
