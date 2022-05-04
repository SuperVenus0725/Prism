use crate::msg::{ConfigResponse, LaunchConfig};
use cosmwasm_std::{Addr, Decimal, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");

pub const TOTAL_TOKEN: Item<Uint128> = Item::new("total_token");
pub const TOTAL_DEPOSIT: Item<Uint128> = Item::new("total_deposit");
pub const DEPOSITS: Map<&Addr, DepositInfo> = Map::new("deposits");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub operator: Addr,
    pub receiver: Addr,
    pub token: Addr,
    pub launch_config: Option<LaunchConfig>,
    pub base_denom: String,
    pub tokens_released: bool,
    pub host_portion: Decimal,
    pub host_portion_receiver: Addr,
}

impl Config {
    pub fn as_res(&self) -> StdResult<ConfigResponse> {
        let res = ConfigResponse {
            operator: self.operator.to_string(),
            receiver: self.receiver.to_string(),
            token: self.token.to_string(),
            launch_config: self.launch_config.clone(),
            base_denom: self.base_denom.clone(),
            tokens_released: self.tokens_released,
            host_portion: self.host_portion,
            host_portion_receiver: self.host_portion_receiver.to_string(),
        };
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct DepositInfo {
    pub amount: Uint128,
    pub withdrew_phase2: bool,
    pub tokens_claimed: bool,
}
