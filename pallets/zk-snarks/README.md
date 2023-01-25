# ZK-Snarks pallet

## Overview

Pallet is designed to store on-chain data:
* **public inputs** - type of `u32`.
* **verification key** - bounded vector of `u8` (max size 1024).
* **proof** - bounded vector of `u8` (max size 1024).

Pallets defines two extrinsics:
* **setup_verification** - allows to store the `public inputs` and the `verification key`.
* **verify** - accepts the `proof` and run the verification procedure.

We can use them to run a Groth16 verification process.

## Build and run
```
cargo run --manifest-path=../../Cargo.toml --release -- --dev
```

Interaction with the node can be done through [polkadotjs](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) app. First, change the configuration of the polkadotjs to point to your local node.
Then navigate to `Extrinsics` panel (*Developer -> Extrinsics*). 

<center>
    
![Extrinsics](https://github.com/bright/zk-snarks-with-substrate/blob/main/blog/img/extrinsicse_tab.png)
    
</center> 

In the field `submit the following extrinsic`, please select `zkSnarks`. Fill in the fields as shown in the image below. Data for `vecKey` can be found under `blog/data/verification_key.json`.

<center>

![Setup Verification](https://github.com/bright/zk-snarks-with-substrate/blob/main/blog/img/vk.png)
    
</center> 

To upload data on blockchain, please press the `Submit Transaction`. Next, we will switch to the second extrinsic `verify` and we will upload a `blog/data/proof.json` file.

<center>
    
![Verify](https://github.com/bright/zk-snarks-with-substrate/blob/main/blog/img/proof.png)
    
</center> 

Finally, we should get the result:

<center>
    
![Result](https://github.com/bright/zk-snarks-with-substrate/blob/main/blog/img/verification_success.png)
    
</center> 
## Unit tests:
```
cargo test --manifest-path=../../Cargo.toml
```