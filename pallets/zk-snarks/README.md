# ZK-Snarks pallet

## Overview

**Please note: This version is just a skeleton for the further implementation.**

Pallet is designed to store on-chain data:
* **public inputs** - type of `u32`.
* **verification key** - bounded vector of `u8` (max size 1024).
* **proof** - bounded vector of `u8` (max size 1024).

Pallets defines two extrinsics:
* **setup_verification** - allows to store the `public inputs` and the `verification key`.
* **verify** - accepts the `proof` and run the verification procedure.

Currently, verification process is very simple. If the `proof` length is equal to the `public inputs` value, than verification pass. Otherwise, it will fail.

## Build and run
```
cargo run --manifest-path=../../Cargo.toml --release
```

Interaction with the node can be done through [polkadotjs](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) app. 

## Unit tests:
```
cargo test --manifest-path=../../Cargo.toml
```