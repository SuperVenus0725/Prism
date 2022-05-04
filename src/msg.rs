use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LaunchConfig {
    pub amount: Uint128,
    // pahse 1: can deposit and withdraw
    pub phase1_start: u64,
    // phase2: can withdraw one time. Allowed withdraw decreases 100% to 0% over time.
    pub phase2_start: u64,
    pub phase2_end: u64,
    // time in seconds for each slot in phase2
    pub phase2_slot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub operator: String,
    pub receiver: String,
    pub token: String,
    pub base_denom: String,
    pub host_portion: Decimal,
    pub host_portion_receiver: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {},
    Withdraw { amount: Option<Uint128> },
    WithdrawTokens {},
    PostInitialize { launch_config: LaunchConfig },
    AdminWithdraw {},
    ReleaseTokens {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    DepositInfo { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub operator: String,
    pub receiver: String,
    pub token: String,
    pub launch_config: Option<LaunchConfig>,
    pub base_denom: String,
    pub tokens_released: bool,
    pub host_portion: Decimal,
    pub host_portion_receiver: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositResponse {
    pub deposit: Uint128,
    pub total_deposit: Uint128,
    pub withdrawable_amount: Uint128,
    pub tokens_to_claim: Uint128,
    pub can_claim: bool,
}
