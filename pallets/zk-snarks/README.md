# ZK-Snarks pallet

## Overview

**Please note: This version is just a skeleton for the further implementation.**

Pallet is designed to store on-chain data:
* **public inputs** - type of `u32`.
* **verification key** - bounded vector of `u8` (max size 1024).
* **proof** - bounded vector of `u8` (max size 1024).

Pallets define two extrinsics:
* **setup_verification** - allows storing the `public inputs` and the `verification key`.
* **verify** - accepts the `proof` and runs the verification procedure.

Currently, the verification process is very simple. If the `proof` length is equal to the `public inputs` value than verification pass. Otherwise, it will fail.

## Build and run
```
cargo run --manifest-path=../../Cargo.toml --release -- --dev
```

Interaction with the node can be done through [polkadotjs](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) app. First, change the configuration of the polkadotjs to point to your local node.
Then navigate to `Extrinsics` panel (*Developer -> Extrinsics*). 

<center>
    
![Extrinsics](https://github.com/bright/zk-snarks-with-substrate/blob/M1/pallets/zk-snarks/sample/panel.png)
    
</center> 

In the field `submit the following extrinsic`, please select `zkSnarks`. Fill in the fields as shown in the image below. Data for `vecKey` can be found under `pallets/zk-snarks/sample/vk.json`.

<center>
    
![Setup Verification](https://github.com/bright/zk-snarks-with-substrate/blob/M1/pallets/zk-snarks/sample/vk.png)
    
</center> 

To upload data on blockchain, please press the `Submit Transaction`. Next, we will switch to the second extrinsic `verify` and we will upload a `pallets/zk-snarks/sample/proof.json` file.

<center>
    
![Verify](https://github.com/bright/zk-snarks-with-substrate/blob/M1/pallets/zk-snarks/sample/proof.png)
    
</center> 

Finally, we should get the result:

<center>
    
![Result](https://github.com/bright/zk-snarks-with-substrate/blob/M1/pallets/zk-snarks/sample/result.png)
    
</center> 

## Unit tests:
```
cargo test --manifest-path=../../Cargo.toml
```