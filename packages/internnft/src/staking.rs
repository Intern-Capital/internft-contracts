use cosmwasm_std::CanonicalAddr;
use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    nft_contract_addr: CanonicalAddr,
    owner: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows this contract to be on the receiving end of a SendNft{contract, token_id, msg} call
    /// to the nft contract. The same thing as sending CW721 tokens to a contract.
    Receive(Cw721ReceiveMsg),
    /// Allows the calling user to withdraw the specified nft if they own it.
    WithdrawNft { nft_id: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw721HookMsg {
    /// Just an example for now.
    StakeNft {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
