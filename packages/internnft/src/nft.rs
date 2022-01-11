use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Coin, StdError, StdResult};
use cw721::{Expiration, OwnerOfResponse};
use cw721_base::msg::{ExecuteMsg as CW721ExecuteMsg, QueryMsg as CW721QueryMsg};
use cw721_base::state::Approval;

// ----------------- begin CW721 ^0.9.2 shim ----------------- //

// adapted from: https://github.com/CosmWasm/cw-nfts/blob/5e1e72a3682f988d4504b94f2e203dd4a5a99ad9/contracts/cw721-metadata-onchain/src/lib.rs#L7-L26
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Cw721Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Cw721Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Cw721Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

// adapted from: https://github.com/CosmWasm/cw-nfts/blob/5e1e72a3682f988d4504b94f2e203dd4a5a99ad9/packages/cw721/src/query.rs#L93-L109
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Cw721NftInfoResponse {
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// You can add any custom metadata here when you extend cw721-base
    pub extension: Cw721Metadata,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Cw721AllNftInfoResponse {
    /// Who can transfer the token
    pub access: OwnerOfResponse,
    /// Data on the token itself,
    pub info: Cw721NftInfoResponse,
}

// ----------------- end CW721 ^0.9.2 shim----------------- //

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The maximum allowed number of xyz tokens
    pub token_supply: u64,
    /// The maximum number of tokens a particular wallet can hold
    pub wallet_limit: u32,
    /// The price to mint a new xyz (doesn't apply to the contract owner)
    pub mint_fee: Coin,
    //the staking contract that can make changes to gold and exp
    pub staking_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Copy)]
pub struct InternExtension {
    pub experience: u64,
    pub gold: u64,
    pub stamina: u64,
}

impl InternExtension {
    pub fn as_traits(&self) -> Vec<Cw721Trait> {
        vec![
            Cw721Trait {
                display_type: None,
                trait_type: "experience".to_string(),
                value: self.experience.to_string(),
            },
            Cw721Trait {
                display_type: None,
                trait_type: "gold".to_string(),
                value: self.gold.to_string(),
            },
            Cw721Trait {
                display_type: None,
                trait_type: "stamina".to_string(),
                value: self.stamina.to_string(),
            },
        ]
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InternTokenInfo {
    pub owner: Addr,
    pub approvals: Vec<Approval>,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub extension: InternExtension,
}

impl InternTokenInfo {
    pub fn as_cw721_nft_info(&self) -> Cw721NftInfoResponse {
        Cw721NftInfoResponse {
            token_uri: None,
            extension: Cw721Metadata {
                name: Some(self.name.clone()),
                // TODO: Put something for image.
                image: None,
                description: Some(self.description.clone()),
                attributes: Some(self.extension.as_traits()),
                image_data: None,
                external_url: None,
                animation_url: None,
                background_color: None,
                youtube_url: None,
            },
        }
    }
}

pub fn full_token_id(numeric_token_id: String) -> StdResult<String> {
    // make sure the string is an integer
    numeric_token_id
        .parse::<u64>()
        .map_err(|_| StdError::generic_err("expected numeric token identifier"))?;
    Ok(format!("intern #{}", numeric_token_id))
}

pub fn numeric_token_id(full_token_id: String) -> StdResult<String> {
    if !full_token_id.starts_with("intern #") {
        return Err(StdError::generic_err(
            "expected full token identifier, like 'intern #123'",
        ));
    }
    Ok(full_token_id.trim_start_matches("intern #").to_string())
}

/// This overrides the ExecuteMsg enum defined in cw721-base
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub config: Config,
}

/// This overrides the ExecuteMsg enum defined in cw721-base
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {},
    /// Update token minting and supply configuration.
    UpdateConfig {
        config: Config,
    },
    /// Withdraw from current contract balance to owner address.
    Withdraw {
        amount: Vec<Coin>,
    },
    UpdateTrait {
        token_id: String,
        exp: u64,
        gold: u64,
        stamina: u64,
    },
    /// BELOW ARE COPIED FROM CW721-BASE
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
}

impl From<ExecuteMsg> for CW721ExecuteMsg<InternExtension> {
    fn from(msg: ExecuteMsg) -> CW721ExecuteMsg<InternExtension> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => CW721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => CW721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => CW721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::Revoke { spender, token_id } => {
                CW721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                CW721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => CW721ExecuteMsg::RevokeAll { operator },
            _ => panic!("cannot covert {:?} to CW721ExecuteMsg", msg),
        }
    }
}

/// This overrides the ExecuteMsg enum defined in cw721-base
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current contract config
    /// Return type: Config
    Config {},
    /// Returns all tokens owned by the given address, [] if unset.
    /// Return type: InternTokensResponse.
    InternTokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Lists all token_ids controlled by the contract.
    /// Return type: InternTokensResponse.
    AllInternTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract: InternTokenInfo.
    InternNftInfo {
        token_id: String,
    },
    /// Returns the number of tokens owned by the given address
    /// Return type: NumTokensResponse
    NumTokensForOwner {
        owner: String,
    },

    // BELOW ARE COPIED FROM CW721-BASE
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    ApprovedForAll {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

impl From<QueryMsg> for CW721QueryMsg {
    fn from(msg: QueryMsg) -> CW721QueryMsg {
        match msg {
            QueryMsg::InternTokens {
                owner,
                start_after,
                limit,
            } => CW721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllInternTokens { start_after, limit } => {
                CW721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::InternNftInfo { token_id } => CW721QueryMsg::NftInfo { token_id },
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => CW721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            } => CW721QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => CW721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => CW721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => CW721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => CW721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => CW721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                CW721QueryMsg::AllTokens { start_after, limit }
            }
            _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InternTokensResponse {
    pub tokens: Vec<InternTokenInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz_token_info_as_cw721_nft_info() {
        let info = InternTokenInfo {
            name: "xyz #1".to_string(),
            owner: Addr::unchecked("testing owner"),
            description: "testing description".to_string(),
            image: None,
            approvals: vec![],
            extension: InternExtension {
                experience: 10,
                gold: 100,
                stamina: 100,
            },
        };

        assert_eq!(
            info.as_cw721_nft_info(),
            Cw721NftInfoResponse {
                token_uri: None,
                extension: Cw721Metadata {
                    name: Some("xyz #1".to_string()),
                    description: Some("testing description".to_string()),
                    image: None,
                    attributes: Some(vec![
                        Cw721Trait {
                            display_type: None,
                            trait_type: "experience".to_string(),
                            value: "10".to_string(),
                        },
                        Cw721Trait {
                            display_type: None,
                            trait_type: "gold".to_string(),
                            value: "100".to_string(),
                        },
                        Cw721Trait {
                            display_type: None,
                            trait_type: "stamina".to_string(),
                            value: "100".to_string(),
                        },
                    ]),
                    image_data: None,
                    animation_url: None,
                    youtube_url: None,
                    external_url: None,
                    background_color: None
                }
            }
        )
    }
}
