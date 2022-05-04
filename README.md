# Prism Forge

This contract provides functionality for executing a "fair launch" for distribution of the initial PRISM tokens. This consists of a two phase auction for the tokens. During Phase 1, users can deposit and withdraw any amount of uusd. During Phase 2, users can only withdraw tokens. After Phase 2, users can withdraw their pro-rata allocated portion of the distributed PRISM tokens.

## ExecuteMsg:

- **Deposit**: Deposit uusd into this contract, only allowed durin Phase1.
- **Withdraw**: Withdraw uusd into this contract, allowed during Phase1 and Phase2.
- **WithdrawTokens**: Withdraw pro-rata allocated PRISM tokens, only allowed at the end of the launch (after Phase2).
- **PostInitialize**: Initialize the contract's LaunchConfig parameters, which contains the total PRISM distribution amount and the phase start/end timestamps. Must be called by owner.
- **AdminWithdraw**: Withdraw the contract's uusd balance at the end of the launch. Must be called by the operator address.
- **ReleaseTokens**: Allows depositors to claim their share of the tokens. Must be called by the operator address.

## QueryMsg:

- **Config**: Retrives contract configuration paraameters.
- **DepositInfo**: Retrives deposit info for a user, which includes the user's deposit amount and the total deposit amount.

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## Formatting

Make sure you run `rustfmt` before creating a PR to the repo. You need to install the `nightly` version of `rustfmt`.

```sh
rustup toolchain install nightly
```

To run `rustfmt`,

```sh
cargo fmt
```

## Linting

You should run `clippy` also. This is a lint tool for rust. It suggests more efficient/readable code.
You can see [the clippy document](https://rust-lang.github.io/rust-clippy/master/index.html) for more information.
You need to install `nightly` version of `clippy`.

### Install

```sh
rustup toolchain install nightly
```

### Run

```sh
cargo clippy --all --all-targets -- -D warnings
```

## Testing

Developers are strongly encouraged to write unit tests for new code, and to submit new unit tests for old code. Unit tests can be compiled and run with: `cargo test --all`. For more details, please reference [Unit Tests](https://github.com/CodeChain-io/codechain/wiki/Unit-Tests).
