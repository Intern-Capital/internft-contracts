use rsa::{hash::Hash, padding::PaddingScheme, PublicKey};
use serde_json;
use sha2::{Digest, Sha256};

use cosmwasm_std::{
    Attribute, BankMsg, Binary, Coin, DepsMut, Empty, Env, MessageInfo, Order, Response, StdError,
    StdResult, Storage,
};
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw721_base::{msg::ExecuteMsg as Cw721ExecuteMsg, Cw721Contract};
use internnft::nft::{
    full_token_id, numeric_token_id, Config, ExecuteMsg, InstantiateMsg, InternExtension,
    InternTokenInfo, MigrateMsg,
};

use crate::error::ContractError;
use crate::state::{load_captcha_public_key, save_captcha_public_key, tokens, CONFIG, OWNER};

const INTERN: &str = "intern";

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    let cw721_contract = Cw721Contract::<InternExtension, Empty>::default();

    let contract_info = ContractInfoResponse {
        name: INTERN.to_string(),
        symbol: INTERN.to_string(),
    };
    cw721_contract
        .contract_info
        .save(deps.storage, &contract_info)?;

    CONFIG.save(deps.storage, &msg.config)?;
    OWNER.save(deps.storage, &info.sender.to_string())?;

    Ok(Response::default())
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    //TODO: Write minting contract.
    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender))
}

fn check_sufficient_funds(funds: Vec<Coin>, required: Coin) -> Result<(), ContractError> {
    if required.amount.u128() == 0 {
        return Ok(());
    }
    let sent_sufficient_funds = funds.iter().any(|coin| {
        // check if a given sent coin matches denom
        // and has sufficient amount
        coin.denom == required.denom && coin.amount.u128() >= required.amount.u128()
    });
    if sent_sufficient_funds {
        Ok(())
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "insufficient funds sent",
        )))
    }
}

fn check_wallet_limit(
    storage: &dyn Storage,
    owner: cosmwasm_std::Addr,
    limit: u32,
) -> Result<(), ContractError> {
    let num_wallet_tokens = tokens()
        .idx
        .owner
        .prefix(owner)
        .range(storage, None, None, Order::Ascending)
        .count();

    if num_wallet_tokens >= limit as usize {
        Err(ContractError::WalletLimit {})
    } else {
        Ok(())
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    config: Config,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Vec<Coin>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new().add_message(BankMsg::Send {
        amount,
        to_address: owner,
    }))
}

pub fn cw721_base_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let cw721_contract = Cw721Contract::<InternExtension, Empty>::default();
    let cw721_msg: Cw721ExecuteMsg<InternExtension> = msg.into();
    let cw721_msg_full_token_id = match cw721_msg {
        Cw721ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => Cw721ExecuteMsg::Approve {
            spender,
            expires,
            token_id: full_token_id(token_id)?,
        },
        Cw721ExecuteMsg::Revoke { spender, token_id } => Cw721ExecuteMsg::Revoke {
            spender,
            token_id: full_token_id(token_id)?,
        },
        Cw721ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => Cw721ExecuteMsg::TransferNft {
            recipient,
            token_id: full_token_id(token_id)?,
        },
        Cw721ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => Cw721ExecuteMsg::SendNft {
            contract,
            msg,
            token_id: full_token_id(token_id)?,
        },
        _ => cw721_msg,
    };

    let mut response = (match cw721_msg_full_token_id {
        Cw721ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        _ => cw721_contract
            .execute(deps, env, info, cw721_msg_full_token_id)
            .map_err(|err| err.into()),
    })?;

    response.attributes = response
        .attributes
        .iter()
        .map(|attr| {
            if attr.key == "token_id" {
                Attribute::new(
                    "token_id",
                    numeric_token_id(attr.value.to_string()).unwrap(),
                )
            } else {
                attr.clone()
            }
        })
        .collect();
    Ok(response)
}

pub fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let cw721_contract = Cw721Contract::<InternExtension, Empty>::default();
    // Transfer token
    cw721_contract._transfer_nft(deps, &env, &info, &contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: numeric_token_id(token_id.clone())?,
        msg,
    };

    // Send message
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default().add_attribute("action", "migrate"))
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{to_binary, Addr, Timestamp};
    use cw721::{Cw721ReceiveMsg, Expiration};
    use cw721_base::state::Approval;

    const ADDR1: &str = "addr1";
    const ADDR2: &str = "addr2";

    fn token_examples() -> Vec<InternTokenInfo> {
        vec![
            InternTokenInfo {
                owner: Addr::unchecked(ADDR1),
                approvals: vec![],
                name: "xyz #1".to_string(),
                description: "".to_string(),
                image: None,
                extension: InternExtension {
                    coordinates: Coordinates { x: 1, y: 1, z: 1 },
                    arrival: Timestamp::from_nanos(0),
                    prev_coordinates: None,
                },
            },
            InternTokenInfo {
                owner: Addr::unchecked(ADDR2),
                approvals: vec![],
                name: "xyz #2".to_string(),
                description: "".to_string(),
                image: None,
                extension: InternExtension {
                    coordinates: Coordinates { x: 2, y: 2, z: 2 },
                    arrival: Timestamp::from_nanos(0),
                    prev_coordinates: None,
                },
            },
        ]
    }

    fn setup_storage(deps: DepsMut) {
        for token in token_examples().iter() {
            tokens().save(deps.storage, &token.name, token).unwrap();
        }
    }

    fn numeric_id_error() -> ContractError {
        ContractError::Std(StdError::generic_err("expected numeric token identifier"))
    }

    #[test]
    fn cw721_transfer() {
        let mut deps = mock_dependencies(&[]);
        setup_storage(deps.as_mut());

        // blocks full token identifiers
        let err = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::TransferNft {
                recipient: ADDR2.to_string(),
                token_id: "xyz #1".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, numeric_id_error());

        // transfer xyz #1
        let res = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::TransferNft {
                recipient: ADDR2.to_string(),
                token_id: "1".to_string(),
            },
        )
        .unwrap();

        // ensure response event emits the transferred token_id
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "token_id" && attr.value == "1"));

        // check ownership was updated
        let token = tokens().load(&deps.storage, "xyz #1").unwrap();
        assert_eq!(token.name, "xyz #1");
        assert_eq!(token.owner.to_string(), ADDR2.to_string());
    }

    #[test]
    fn cw721_approve_revoke() {
        let mut deps = mock_dependencies(&[]);
        setup_storage(deps.as_mut());

        // approve blocks full token identifiers
        let err = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::Approve {
                spender: ADDR2.to_string(),
                token_id: "xyz #1".to_string(),
                expires: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, numeric_id_error());

        // grant an approval
        let res = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::Approve {
                spender: ADDR2.to_string(),
                token_id: "1".to_string(),
                expires: None,
            },
        )
        .unwrap();

        // ensure response event emits the transferred token_id
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "token_id" && attr.value == "1"));

        // check approval was added
        let token = tokens().load(&deps.storage, "xyz #1").unwrap();
        assert_eq!(token.name, "xyz #1");
        assert_eq!(
            token.approvals,
            vec![Approval {
                spender: Addr::unchecked(ADDR2),
                expires: Expiration::Never {}
            }]
        );

        // revoke blocks full token identifiers
        let err = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::Revoke {
                spender: ADDR2.to_string(),
                token_id: "xyz #1".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, numeric_id_error());

        // revoke the approval
        let res = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::Revoke {
                spender: ADDR2.to_string(),
                token_id: "1".to_string(),
            },
        )
        .unwrap();

        // ensure response event emits the transferred token_id
        assert!(res
            .attributes
            .iter()
            .any(|attr| attr.key == "token_id" && attr.value == "1"));

        // check approval was revoked
        let token = tokens().load(&deps.storage, "xyz #1").unwrap();
        assert_eq!(token.name, "xyz #1");
        assert_eq!(token.approvals, vec![]);
    }

    #[test]
    fn cw721_send_nft() {
        let mut deps = mock_dependencies(&[]);
        setup_storage(deps.as_mut());

        let token_id = "1".to_string();
        let target = "another_contract".to_string();
        let msg = to_binary("my msg").unwrap();

        // blocks full token identifiers
        let err = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::SendNft {
                contract: target.clone(),
                token_id: "xyz #1".to_string(),
                msg: msg.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, numeric_id_error());

        // send a token to a contract
        let res = cw721_base_execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            ExecuteMsg::SendNft {
                contract: target.clone(),
                token_id: token_id.clone(),
                msg: msg.clone(),
            },
        )
        .unwrap();

        let payload = Cw721ReceiveMsg {
            sender: ADDR1.to_string(),
            token_id: token_id.clone(),
            msg,
        };
        let expected = payload.into_cosmos_msg(target).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_message(expected)
                .add_attribute("action", "send_nft")
                .add_attribute("sender", ADDR1)
                .add_attribute("recipient", "another_contract")
                .add_attribute("token_id", token_id)
        );
    }
}
