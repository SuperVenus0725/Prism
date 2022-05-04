use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MemoryStorage, MessageInfo, OwnedDeps, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::contract::{deposit, execute, instantiate, query, release_tokens};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, DepositResponse, ExecuteMsg, InstantiateMsg, LaunchConfig, QueryMsg,
};

const SECONDS_PER_HOUR: u64 = 60 * 60;

pub fn init(deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>) {
    let msg = InstantiateMsg {
        operator: "owner0001".to_string(),
        receiver: "receiver0000".to_string(),
        token: "prism0001".to_string(),
        base_denom: "uusd".to_string(),
        host_portion: Decimal::zero(),
        host_portion_receiver: "host0000".to_string(),
    };

    let info = mock_info("owner0001", &[]);
    let env = mock_env();
    instantiate(deps.as_mut(), env, info, msg).unwrap();
}

pub fn post_init(deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>) {
    let info = mock_info("owner0001", &[]);
    let env = mock_env();
    let launch_config = LaunchConfig {
        amount: Uint128::from(1_000_000u64),
        phase1_start: env.block.time.seconds(),
        phase2_start: env.block.time.seconds() + 100,
        phase2_end: env.block.time.seconds() + 100 + SECONDS_PER_HOUR,
        phase2_slot_period: SECONDS_PER_HOUR,
    };
    do_post_initialize(deps.as_mut(), env, info, launch_config).unwrap();
}

pub fn do_deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::Deposit {})
}

pub fn do_post_initialize(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    launch_config: LaunchConfig,
) -> Result<Response, ContractError> {
    execute(
        deps,
        env,
        info,
        ExecuteMsg::PostInitialize { launch_config },
    )
}

pub fn do_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::Withdraw { amount })
}

pub fn do_admin_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::AdminWithdraw {})
}

pub fn do_withdraw_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::WithdrawTokens {})
}

pub fn do_release_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    execute(deps, env, info, ExecuteMsg::ReleaseTokens {})
}

pub fn do_query_deposit_info(deps: Deps, env: Env, address: String) -> StdResult<DepositResponse> {
    from_binary(&query(deps, env, QueryMsg::DepositInfo { address }).unwrap())
}

#[test]
fn proper_initialize() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        operator: "owner0001".to_string(),
        receiver: "receiver0000".to_string(),
        token: "prism0001".to_string(),
        base_denom: "uusd".to_string(),
        host_portion: Decimal::percent(110),
        host_portion_receiver: "host0000".to_string(),
    };

    let info = mock_info("owner0001", &[]);
    let env = mock_env();
    let err = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidHostPortion {})
}

#[test]
fn proper_post_initialize() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);

    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            operator: "owner0001".to_string(),
            receiver: "receiver0000".to_string(),
            token: "prism0001".to_string(),
            launch_config: None,
            base_denom: "uusd".to_string(),
            tokens_released: false,
            host_portion: Decimal::zero(),
            host_portion_receiver: "host0000".to_string(),
        }
    );

    let mut info = mock_info("", &[]);
    let env = mock_env();
    let mut launch_config = LaunchConfig {
        amount: Uint128::from(1_000_000u64),
        phase1_start: env.block.time.seconds(),
        phase2_start: env.block.time.seconds() + 100,
        phase2_end: env.block.time.seconds() + 100 + SECONDS_PER_HOUR,
        phase2_slot_period: SECONDS_PER_HOUR,
    };

    // unauthorized
    let err = do_post_initialize(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        launch_config.clone(),
    );
    assert_eq!(err.unwrap_err(), ContractError::Unauthorized {});

    // invalid launch config (phase1 start in the past)
    info.sender = Addr::unchecked("owner0001");
    launch_config.phase1_start = env.block.time.seconds() - 100;
    let err = do_post_initialize(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        launch_config.clone(),
    );
    assert_eq!(err.unwrap_err(), ContractError::InvalidLaunchConfig {});

    // invalid launch config (phase 2 length less than 1 hour)
    info.sender = Addr::unchecked("owner0001");
    launch_config.phase1_start = env.block.time.seconds();
    launch_config.phase2_end = launch_config.phase2_start + 1;
    let err = do_post_initialize(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        launch_config.clone(),
    );
    assert_eq!(err.unwrap_err(), ContractError::InvalidLaunchConfig {});

    // invalid launch config (slot period is zero)
    info.sender = Addr::unchecked("owner0001");
    launch_config.phase2_slot_period = 0u64;
    let err = do_post_initialize(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        launch_config.clone(),
    );
    assert_eq!(err.unwrap_err(), ContractError::InvalidLaunchConfig {});

    // success
    launch_config.phase1_start = env.block.time.seconds();
    launch_config.phase2_end = env.block.time.seconds() + 100 + SECONDS_PER_HOUR;
    launch_config.phase2_slot_period = SECONDS_PER_HOUR;
    let res = do_post_initialize(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        launch_config.clone(),
    );
    assert!(res.is_ok());
    let msgs = res.unwrap().messages;
    assert_eq!(msgs.len(), 1);
    let msg = &msgs[0].msg;
    match msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds: _,
        }) => {
            assert_eq!(contract_addr, "prism0001");
            assert_eq!(
                msg,
                &to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: "owner0001".to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: Uint128::from(1_000_000u64)
                })
                .unwrap()
            )
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            operator: "owner0001".to_string(),
            receiver: "receiver0000".to_string(),
            token: "prism0001".to_string(),
            launch_config: Some(launch_config.clone()),
            base_denom: "uusd".to_string(),
            tokens_released: false,
            host_portion: Decimal::zero(),
            host_portion_receiver: "host0000".to_string(),
        }
    );

    let err = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::PostInitialize { launch_config },
    );
    assert_eq!(err.unwrap_err(), ContractError::DuplicatePostInit {});
}

#[test]
fn proper_deposit() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);
    post_init(&mut deps);

    let mut info = mock_info("addr0001", &[]);
    let mut env = mock_env();

    // failed deposit, before phase 1
    env.block.time = env.block.time.minus_seconds(150u64);
    let err = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(
        err.unwrap_err(),
        ContractError::InvalidDeposit {
            reason: "deposit period did not start yet".to_string()
        }
    );

    // error, no coins sent with deposit
    env.block.time = env.block.time.plus_seconds(150u64);
    let err = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(
        err.unwrap_err(),
        ContractError::InvalidDeposit {
            reason: "requires 1 coin deposited".to_string()
        }
    );

    // error, zero amount
    info.funds = vec![Coin::new(0, "uusd")];
    let err = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(
        err.unwrap_err(),
        ContractError::InvalidDeposit {
            reason: "requires uusd and positive amount".to_string()
        }
    );

    // error, wrong currency
    info.funds = vec![Coin::new(0, "ukrw")];
    let err = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(
        err.unwrap_err(),
        ContractError::InvalidDeposit {
            reason: "requires uusd and positive amount".to_string()
        }
    );

    // successful deposit
    info.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    // query deposit responses for addr0001
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(1_000u128),
            total_deposit: Uint128::from(1_000u128),
            withdrawable_amount: Uint128::from(1_000u128),
            tokens_to_claim: Uint128::from(1_000_000u64),
            can_claim: false,
        }
    );

    // query deposit responses for addr0002
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "addr0002".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::zero(),
            total_deposit: Uint128::from(1_000u128),
            withdrawable_amount: Uint128::zero(),
            tokens_to_claim: Uint128::zero(),
            can_claim: false,
        }
    );

    // failed deposit, after phase 1
    env.block.time = env.block.time.plus_seconds(150u64);
    let err = do_deposit(deps.as_mut(), env, info);
    assert_eq!(
        err.unwrap_err(),
        ContractError::InvalidDeposit {
            reason: "deposit period is over".to_string()
        }
    );
}

#[test]
fn proper_withdraw() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);
    post_init(&mut deps);

    let mut info = mock_info("addr0001", &[]);
    let mut env = mock_env();

    // successful deposit
    info.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    // try to withdraw 0, expect error
    let err = do_withdraw(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(Uint128::zero()),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdraw {
            reason: "withdraw amount must be bigger than 0".to_string()
        }
    );

    // successful withdraw 100
    let res = do_withdraw(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(Uint128::from(100u128)),
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, info.sender.as_str());
            assert_eq!(
                amount[0],
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(100u128)
                }
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    // query deposit responses for addr0001
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(900u128),
            total_deposit: Uint128::from(900u128),
            withdrawable_amount: Uint128::from(900u128),
            tokens_to_claim: Uint128::from(1_000_000u64),
            can_claim: false,
        }
    );

    // successful withdraw remaining, we have 900 left
    let res = do_withdraw(deps.as_mut(), env.clone(), info.clone(), None).unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, info.sender.as_str());
            assert_eq!(
                amount[0],
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(900u128)
                }
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    // query deposit responses for addr0001
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::zero(),
            total_deposit: Uint128::zero(),
            withdrawable_amount: Uint128::zero(),
            tokens_to_claim: Uint128::zero(),
            can_claim: false,
        }
    );

    let err = do_withdraw(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(Uint128::from(1_000u128)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdraw {
            reason: "no funds available to withdraw".to_string()
        }
    );

    // successful deposit again
    info.funds = vec![Coin::new(200_000_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    // execute another withdraw
    let res = do_withdraw(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(Uint128::from(101_000_000u128)),
    )
    .unwrap();
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, info.sender.as_str());
            assert_eq!(
                amount[0],
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(101_000_000u128) // 101_000_000
                }
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    // fast forward to after phase 2, withdraws not allowed
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);
    let err = do_withdraw(
        deps.as_mut(),
        env.clone(),
        info,
        Some(Uint128::from(100u128)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdraw {
            reason: "withdraw period is over".to_string()
        }
    );

    // withdrawable amount should be zero
    let deposit_info = do_query_deposit_info(deps.as_ref(), env, "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(99_000_000u128),
            total_deposit: Uint128::from(99_000_000u128),
            withdrawable_amount: Uint128::zero(),
            tokens_to_claim: Uint128::from(1_000_000u64),
            can_claim: false, // phase 2 is over, but tokens not released, so cant claim yet
        }
    );
}

#[test]
fn proper_withdraw_phase3() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);

    let env = mock_env();
    let info = mock_info("owner0001", &[]);
    let launch_config = LaunchConfig {
        amount: Uint128::from(1_000_000u64),
        phase1_start: env.block.time.seconds(),
        phase2_start: env.block.time.seconds() + 100,
        phase2_end: env.block.time.seconds() + 100 + 24 * SECONDS_PER_HOUR, // 24 hour phase 2
        phase2_slot_period: SECONDS_PER_HOUR,
    };
    do_post_initialize(deps.as_mut(), mock_env(), info, launch_config).unwrap();

    let mut alice_info = mock_info("alice0000", &[]);
    let mut bob_info = mock_info("bob0000", &[]);
    let mut cindy_info = mock_info("cindy0000", &[]);
    let mut env = mock_env();

    // successful deposit with 3 accounts
    alice_info.funds = vec![Coin::new(100_000_000, "uusd")];
    deposit(deps.as_mut(), env.clone(), alice_info.clone()).unwrap();
    bob_info.funds = vec![Coin::new(100_000_000, "uusd")];
    deposit(deps.as_mut(), env.clone(), bob_info.clone()).unwrap();
    cindy_info.funds = vec![Coin::new(100_000_000, "uusd")];
    deposit(deps.as_mut(), env.clone(), cindy_info).unwrap();

    // fast forward to phase 2
    env.block.time = env.block.time.plus_seconds(101);

    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "alice0000".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(100_000_000u128),
            total_deposit: Uint128::from(300_000_000u128),
            withdrawable_amount: Uint128::from(100_000_000u128),
            tokens_to_claim: Uint128::from(333333u128),
            can_claim: false,
        }
    );

    // try to withdraw more than withdrawable
    let err = do_withdraw(
        deps.as_mut(),
        env.clone(),
        alice_info.clone(),
        Some(Uint128::from(100_000_001u128)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdraw {
            reason: "can not withdraw more than current withdrawable amount (100000000)"
                .to_string()
        }
    );

    // valid withdraw
    let res = do_withdraw(
        deps.as_mut(),
        env.clone(),
        alice_info.clone(),
        Some(Uint128::from(1_000_000u128)),
    )
    .unwrap();
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, alice_info.sender.as_str());
            assert_eq!(
                amount[0],
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(1_000_000u128)
                }
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    // try to withdraw again, expect error
    let err = do_withdraw(
        deps.as_mut(),
        env.clone(),
        alice_info,
        Some(Uint128::from(1_000u128)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdraw {
            reason: "a withdraw was already executed on phase 2".to_string()
        }
    );

    // fast forward 6 hours
    env.block.time = env.block.time.plus_seconds(SECONDS_PER_HOUR * 6);
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "bob0000".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(100_000_000u128),
            total_deposit: Uint128::from(299_000_000u128),
            withdrawable_amount: Uint128::from(75_000_000u128), // 100M * 18/24 70833333
            tokens_to_claim: Uint128::from(334448u128),         // 100000000 / 299000000 * 1000000
            can_claim: false,
        }
    );
    // valid withdraw all remaining
    let res = do_withdraw(deps.as_mut(), env.clone(), bob_info.clone(), None).unwrap();
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, bob_info.sender.as_str());
            assert_eq!(
                amount[0],
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(75_000_000u128) // 75_000_000
                }
            );
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "bob0000".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(25000000u128), // 100000000 - 75000000
            total_deposit: Uint128::from(224000000u128),
            withdrawable_amount: Uint128::zero(), // can not withraw more, only one time
            tokens_to_claim: Uint128::from(111607u128), // 25000000 / 224000000 * 1000000
            can_claim: false,
        }
    );

    // last slot of phase 2
    env.block.time = env.block.time.plus_seconds(SECONDS_PER_HOUR * 17 + 3598);
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "cindy0000".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(100000000u128),
            total_deposit: Uint128::from(224000000u128),
            withdrawable_amount: Uint128::from(4166666u128), // 100000000 * 1 / 24
            tokens_to_claim: Uint128::from(446428u128),      // 100000000 / 224000000 * 1000000
            can_claim: false,
        }
    );

    // after phase 2
    env.block.time = env.block.time.plus_seconds(1);
    let deposit_info = do_query_deposit_info(deps.as_ref(), env, "cindy0000".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(100000000u128),
            total_deposit: Uint128::from(224000000u128),
            withdrawable_amount: Uint128::zero(), // 100000000 * 0 / 24
            tokens_to_claim: Uint128::from(446428u128), // 100000000 / 224000000 * 1000000
            can_claim: false,                     // tokens not released, cant claim tokens yet
        }
    );
}

#[test]
fn proper_withdraw_tokens1() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);
    post_init(&mut deps);

    let mut info = mock_info("addr0001", &[]);
    let mut env = mock_env();

    // successful deposit
    info.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    let err = do_withdraw_tokens(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdrawTokens {
            reason: "cannot withdraw tokens yet".to_string()
        }
    );

    // invalid release - prior to phase 2 end
    let owner_info = mock_info("owner0001", &[]);
    let err = do_release_tokens(deps.as_mut(), env.clone(), owner_info.clone()).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidReleaseTokens {
            reason: "cannot release tokens yet".to_string()
        }
    );

    // fast forward past phase 2
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);

    // tokens have not yet been released, so should still return error
    let err = do_withdraw_tokens(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdrawTokens {
            reason: "cannot withdraw tokens yet".to_string()
        }
    );

    // admin executes release tokens --  unauthorized attempt
    let err = do_release_tokens(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // valid attempt
    release_tokens(deps.as_mut(), env.clone(), owner_info.clone()).unwrap();

    // now users can withdraw tokens
    let res = do_withdraw_tokens(deps.as_mut(), env.clone(), info).unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) => {
            assert_eq!(contract_addr, &"prism0001".to_string());
            assert_eq!(
                msg,
                &to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0001".to_string(),
                    amount: Uint128::from(1_000_000u64),
                })
                .unwrap(),
            );
            assert_eq!(funds, &[])
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert!(config_response.tokens_released);

    // invalid withdraw - user hasn't deposited anything
    let info_no_deposit_user = mock_info("addr0002", &[]);
    let err = do_withdraw_tokens(deps.as_mut(), env.clone(), info_no_deposit_user).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdrawTokens {
            reason: "deposit information not found".to_string()
        }
    );

    // invalid release - already released
    let err = do_release_tokens(deps.as_mut(), env, owner_info).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidReleaseTokens {
            reason: "tokens are already released".to_string()
        }
    );
}

#[test]
fn proper_withdraw_tokens2() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);
    post_init(&mut deps);

    let mut info1 = mock_info("addr0001", &[]);
    let mut info2 = mock_info("addr0002", &[]);
    let mut env = mock_env();

    // successful deposit
    info1.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info1.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    info2.funds = vec![Coin::new(5_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info2.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    // fast forward past phase 2
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);

    // admin releases tokens, now users can claim
    let owner_info = mock_info("owner0001", &[]);
    release_tokens(deps.as_mut(), env.clone(), owner_info).unwrap();

    // addr0001 gets 1M * 1/6
    let res = do_withdraw_tokens(deps.as_mut(), env.clone(), info1.clone()).unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) => {
            assert_eq!(contract_addr, &"prism0001".to_string());
            assert_eq!(
                msg,
                &to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0001".to_string(),
                    amount: Uint128::from(1_000_000u64).multiply_ratio(1u128, 6u128),
                })
                .unwrap(),
            );
            assert_eq!(funds, &[])
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    // addr0001 try to withdraw again, expect error
    let err = do_withdraw_tokens(deps.as_mut(), env.clone(), info1).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdrawTokens {
            reason: "tokens were already claimed".to_string()
        }
    );

    // addr0002 gets 1M * 5/6
    let res = do_withdraw_tokens(deps.as_mut(), env, info2).unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) => {
            assert_eq!(contract_addr, &"prism0001".to_string());
            assert_eq!(
                msg,
                &to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0002".to_string(),
                    amount: Uint128::from(1_000_000u64).multiply_ratio(5u128, 6u128),
                })
                .unwrap(),
            );
            assert_eq!(funds, &[])
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };
}

#[test]
fn test_no_tokens_for_withdraw() {
    let mut deps = mock_dependencies(&[]);
    init(&mut deps);
    post_init(&mut deps);

    let mut info = mock_info("addr0001", &[]);
    let mut info2 = mock_info("addr0002", &[]);
    let mut env = mock_env();

    // successful deposit
    info.funds = vec![Coin::new(10_000_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    info2.funds = vec![Coin::new(1, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info2.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    // fast forward past phase 2
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);

    // release tokens
    let owner_info = mock_info("owner0001", &[]);
    release_tokens(deps.as_mut(), env.clone(), owner_info).unwrap();

    // now users can withdraw tokens
    let res = do_withdraw_tokens(deps.as_mut(), env.clone(), info).unwrap();
    assert_eq!(res.messages.len(), 1);
    let msg = &res.messages[0].msg;
    match msg {
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) => {
            assert_eq!(contract_addr, &"prism0001".to_string());
            assert_eq!(
                msg,
                &to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: "addr0001".to_string(),
                    amount: Uint128::from(999_999u64),
                })
                .unwrap(),
            );
            assert_eq!(funds, &[])
        }
        _ => panic!("Unexpected message: {:?}", msg),
    };

    let err = do_withdraw_tokens(deps.as_mut(), env, info2).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidWithdrawTokens {
            reason: "no tokens available for withdraw".to_string()
        }
    );
}

#[test]
fn proper_admin_withdraw() {
    let mut deps = mock_dependencies(&[]);

    init(&mut deps);
    post_init(&mut deps);

    let mut info1 = mock_info("addr0001", &[]);
    let mut info2 = mock_info("addr0002", &[]);
    let mut env = mock_env();

    // successful deposits -- total 6,000 uusd
    info1.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info1.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    info2.funds = vec![Coin::new(5_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info2);
    assert_eq!(res.unwrap().messages.len(), 0);

    // update contract balance after deposits
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin::new(6_000, "uusd")],
    );

    // admin tries to withdraw before phase 3, expect error
    let owner_info = mock_info("owner0001", &[]);
    let err = do_admin_withdraw(deps.as_mut(), env.clone(), owner_info.clone()).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidAdminWithdraw {
            reason: "cannot withdraw funds yet".to_string()
        }
    );

    // fast forward past phase 2
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);

    // now admin can withdraw uusd -- unauthorized attempt
    let err = do_admin_withdraw(deps.as_mut(), env.clone(), info1).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // valid attempt
    let res = do_admin_withdraw(deps.as_mut(), env.clone(), owner_info.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "admin_withdraw"),
            attr("total_withdraw_amount", "6000"),
            attr("host_amount", "0"),
            attr("remaining_amount", "6000"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "receiver0000".to_string(), // receiver address specified on contract instantiation
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(6000u128), // 6000
            }],
        }))]
    );

    // check that users can not claim yet, even after withdraw admin
    let deposit_info = do_query_deposit_info(deps.as_ref(), env.clone(), "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(1_000u128),
            total_deposit: Uint128::from(6_000u128),
            withdrawable_amount: Uint128::zero(), // can not withdraw on phase 3
            tokens_to_claim: Uint128::from(166666u128), // 1000000 * 1000 / 6000
            can_claim: false,                     // tokens not released, cant claim tokens yet
        }
    );

    // admin releases tokens, now users can claim
    release_tokens(deps.as_mut(), env.clone(), owner_info).unwrap();

    let deposit_info = do_query_deposit_info(deps.as_ref(), env, "addr0001".to_string());
    assert_eq!(
        deposit_info.unwrap(),
        DepositResponse {
            deposit: Uint128::from(1_000u128),
            total_deposit: Uint128::from(6_000u128),
            withdrawable_amount: Uint128::zero(), // can not withdraw on phase 3
            tokens_to_claim: Uint128::from(166666u128), // 1000000 * 1000 / 6000
            can_claim: true,                      // now users can claim tokens
        }
    );
}

#[test]
fn proper_admin_withdraw_with_host() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        operator: "owner0001".to_string(),
        receiver: "receiver0000".to_string(),
        token: "prism0001".to_string(),
        base_denom: "uusd".to_string(),
        host_portion: Decimal::percent(10), // 10% host portion
        host_portion_receiver: "host0000".to_string(),
    };

    let owner_info = mock_info("owner0001", &[]);
    let mut env = mock_env();
    instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

    post_init(&mut deps);

    let mut info1 = mock_info("addr0001", &[]);
    let mut info2 = mock_info("addr0002", &[]);

    // successful deposits -- total 6,000 uusd
    info1.funds = vec![Coin::new(1_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info1.clone());
    assert_eq!(res.unwrap().messages.len(), 0);

    info2.funds = vec![Coin::new(5_000, "uusd")];
    let res = do_deposit(deps.as_mut(), env.clone(), info2);
    assert_eq!(res.unwrap().messages.len(), 0);

    // update contract balance after deposits
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin::new(6_000, "uusd")],
    );

    // fast forward past phase 2
    env.block.time = env.block.time.plus_seconds(100 + SECONDS_PER_HOUR);

    // now admin can withdraw uusd -- unauthorized attempt
    let err = do_admin_withdraw(deps.as_mut(), env.clone(), info1).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // valid attempt
    let res = do_admin_withdraw(deps.as_mut(), env, owner_info).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "admin_withdraw"),
            attr("total_withdraw_amount", "6000"),
            attr("host_amount", "600"),
            attr("remaining_amount", "5400"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "host0000".to_string(), // host address specified on contract instantiation
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(600u128), // 600
                }],
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "receiver0000".to_string(), // receiver address specified on contract instantiation
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::from(5400u128), // 5400
                }],
            }))
        ]
    );
}
