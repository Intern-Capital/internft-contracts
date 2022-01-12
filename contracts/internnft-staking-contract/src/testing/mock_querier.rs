use cosmwasm_std::{Addr, Coin, ContractResult, from_binary, from_slice, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, to_binary, WasmQuery};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use internnft::staking::GetRandomResponse;

use terra_cosmwasm::{TerraQueryWrapper};
use internnft::nft::{InternExtension, InternTokenInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetRandomness { round: u64 },
    InternNftInfo {token_id: String},
}

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e.to_string()),
                    request: bin_request.into(),
                })
            }
        };
    self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
           }) => match from_binary(msg).unwrap() {
                QueryMsg::GetRandomness { round: _ } => {
                        SystemResult::Ok(ContractResult::from(to_binary(&GetRandomResponse {
                            randomness: to_binary("yTBW2ubloeFa+ZRh08Jt+4jVQHHGMX4s3j8mTYKc3oQ=").unwrap(),
                            worker: "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string()
                        })))
                },
                QueryMsg::InternNftInfo {token_id} => {
                    SystemResult::Ok(ContractResult::from(to_binary(&InternTokenInfo {
                        owner: Addr::unchecked(String::from("addr000")+token_id.as_str()),
                        approvals: vec![],
                        name: token_id.to_string(),
                        description: "test".to_string(),
                        image: None,
                        extension: InternExtension {
                            experience: 0,
                            gold: 0,
                            stamina: 100
                        }
                    })))
                }
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
        }
    }
}