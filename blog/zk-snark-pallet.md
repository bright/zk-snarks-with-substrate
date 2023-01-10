# Zk-SNARKs with Substrate (Part 3)

This is the last part of a series of articles about zk-SNARKs. If you haven't read the previous ones (Part 1, Part 2), I encourage you to do so now.

In our first article, we defined Bob's problem. He creates a contest, where the first person who solves the equation: 

<center>

$$ x^2+3=12 $$

</center>

will be able to join his Bright Coders union. As you probably remember, Alice was one of his friends who knew the solution. She was only afraid that revealing it loudly, could encourage others to first claim the vacancy. That's why she decided to use zk-SNARKs.

Until this moment, everything that we did assumed Bob and Alice are in the same place. Alice decided to create proof because she didn't want to reveal the solution to anyone. Bob verified it in front of her, so Alice was certain that she was the first who solved the puzzle. But what if they weren't in the same place? What guarantee will Alice have, that her solution was verified first? This problem can be easily solved if we could move it to the blockchain! Information about the winner will be known to everyone, and the verification process will be more transparent.

We are going to use a framework called [Substrate](https://substrate.io/), to create a custom blockchain. It is written in Rust language and was created by the [Parity](https://www.parity.io/). You can find more information about the Substrate in our other [post](https://brightinventions.pl/blog/5-benefits-of-substrate-blockchain). 

In Substrate, the business logic of the blockchain is hidden in the runtime. We can easily change it, by using components called pallets. Substrate allows us not only to modify already defined pallets but also to create a new custom one. In this article, we will create a pallet for zk-SNARKs. Thanks to this, we will be able to run a blockchain for the proof verification process. The pallet will allow us to store Circom and SnarkJS artifacts, it will also use a Groth16 for the proof verification. Finally, we will use our zk-SNARK pallet to help Alice join the Bright Coders union.

## Starting point

We have already learned that zk-SNARKs can be used to prove knowledge of a solution for a problem, without revealing it. We just need to provide proof that can be later verified by someone else. Creation of such proof is done in a couple of stages, where we first convert our problem to R1CS (*Rank-1 Constraint System*) form, and then transform it to the QAP (*Quadratic Arithmetic Program*). For this process, we used Circom and SnarkJS. We used a Groth16 as a proving system for assurance of the encryption. Finally, we were able to implement a Rust library, which used artifacts from Circom and SnarkJS, to validate proof.

In the previous post, we show how we can use Circom and SnarkJS in the verification process. Let's remind us what artifacts in the context of those tools we get:
* **[public.json](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/data/public.json)** - this file contains public input, in our case, it is a `12` value from our equation.
* **[verification_key.json](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/data/verification_key.json)** - file generated from SnarkJS, it contains a verification key, which "signs" our circuits (transformed equation).
* **[proof.json](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/data/proof.json)** - file created by Alice using SnarkJS, which proves her knowledge of solving the equation. 

As you probably remember, at this stage Bob could use a SnarkJS to verify, if the proof is valid, by running this command:
```
snarkjs groth16 verify verification_key.json public.json proof.json
```
He is using a *Groth16* as a proving system. The output is:
```
[INFO]  snarkJS: OK!
```
which means that proof passed the validation. Now we will try to do the same in the Substrate pallet, using our Groth16 code from the previous article.


## Substrate Pallet
First, let's define what we expect from pallet to do. We definitely would like to have on-chain storage of the artifacts generated from Circom and SnarkJS, because they are going to be used in the verification process. We will need to provide functionality for other participants to send and verify their proofs. Finally, we would like to be informed when someone will send us valid proof.

Based on what we said, we can define an interface for our pallet which is in Substrate called an extirices. We are going to define two methods:
* **setup_verification** - this methods allows Bob to send a public inputs (*public.json*) and the verification key (*verification_key.json*).
* **verify** - thanks to this method, Alice (and others) will be able to send their proofs (*proof.json*) and verify them. When the verification succeeds, it will emit an event that is going to be stored on the blockchain.

## Implementation
The implementation we started from the [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template), which is a single-node blockchain that we could run locally in our development environment. We added there a zk-SNARK pallet which uses Groth16 in the verification process. The final result can be found on our [GitHub](https://github.com/bright/zk-snarks-with-substrate). We will now try to present the core components of this code.

Implementation of the zk-SNARK pallet can be found in the [pallets/zk-snarks/src/lib.rs](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/pallets/zk-snarks/src/lib.rs) file. All pallets in Substrate follow the same skeleton pattern based on the macros:

```
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::config]      // 1. config
  #[pallet::event]       // 2. event
  #[pallet::error]       // 3. error
  #[pallet::storage]     // 4. storage
  #[pallet::call]        // 5. call
}
```

Now we are going to take a closer look at the following sections.

### #[pallet::config]
All pallets in Substrate define a trait called `Config`, which needs to be defined under this macro. We can declare here some specific pallet requirements. In our case, for the zk-SNARK we are going to define some constant values:
```
#[pallet::constant]
type MaxPublicInputsLength: Get<u32>;

#[pallet::constant]
type MaxProofLength: Get<u32>;

#[pallet::constant]
type MaxVerificationKeyLength: Get<u32>;
```
As you probably guess, we are going to use those constants for checking the maximum length of our input files.

### #[pallet::event]
This section defines all events that can be emitted from our pallet. We have defined several events here that left information on the blockchain about the progress of the verification. 

```
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    VerificationSetupCompleted,
    VerificationProofSet,
    VerificationSuccess { who: T::AccountId },
    VerificationFailed,
}
```
What is worth mentioning, Rust language allows the definition of different types of enums fields. We take advantage of this feature when we have declared a `VerificationSuccess` event. The event will store an `AccountId`, which represents an account that belongs to the person who submits valid proof.

### #[pallet::error]
Here we define all errors, which our pallet returns when something goes wrong.
```
#[pallet::error]
pub enum Error<T> {
    PublicInputsMismatch,
    TooLongPublicInputs,
    TooLongVerificationKey,
    TooLongProof,
    ProofIsEmpty,
    VerificationKeyIsNotSet,
    MalformedVerificationKey,
    MalformedProof,
    MalformedPublicInputs,
    NotSupportedCurve,
    NotSupportedProtocol,
    ProofVerificationError,
    ProofCreationError,
    VerificationKeyCreationError,
}
```

### #[pallet::storage]
The storage section defines all information that can be stored on the blockchain. In the context of zk-SNARKs, we would like to store artifacts generated by Bob (public inputs, verification key) and Alice (proof).
```
#[pallet::storage]
pub type PublicInputStorage<T: Config> = StorageValue<_, PublicInputsDef<T>, ValueQuery>;

#[pallet::storage]
pub type ProofStorage<T: Config> = StorageValue<_, ProofDef<T>, ValueQuery>;

#[pallet::storage]
pub type VerificationKeyStorage<T: Config> = StorageValue<_, VerificationKeyDef<T>, ValueQuery>;

```

### #[pallet::call]
Final section declarations extrinsics, which is the interface for pallets. As we mentioned earlier, we defined two methods. Thanks to them, we will be able to interact with the zk-SNARK pallet.

The first method is for the verification setup, which is going to be used by Bob to set up the contest. We are going to store public input and the verification key. We will emit a *VerificationSetupCompleted* event. If anything goes wrong, we will return an appropriate error.
```
pub fn setup_verification(
        origin: OriginFor<T>,
        pub_input: Vec<u8>,
        vec_vk: Vec<u8>,
    ) -> DispatchResult {
    let inputs = store_public_inputs::<T>(pub_input)?;
    let vk = store_verification_key::<T>(vec_vk)?;
    
    ensure!(vk.public_inputs_len == inputs.len() as u8, Error::<T>::PublicInputsMismatch);
    Self::deposit_event(Event::<T>::VerificationSetupCompleted);
    Ok(())
}
```

The second one is for Alice (and others), to send and validate their proofs. First, we will get already stored values for the public inputs and the verification key. Then we will store a proof and verify it with our Groth16 library. Depending on the verification result we will send an appropriate event.

```
pub fn verify(origin: OriginFor<T>, vec_proof: Vec<u8>) -> DispatchResult {
    let sender = ensure_signed(origin)?;
    let vk = get_verification_key::<T>()?;
    let inputs = get_public_inputs::<T>()?;

    let proof = store_proof::<T>(vec_proof)?;
    Self::deposit_event(Event::<T>::VerificationProofSet);

    match verify(vk, proof, prepare_public_inputs(inputs)) {
        Ok(true) => {
            Self::deposit_event(Event::<T>::VerificationSuccess { who: sender });
            Ok(())
        },
        Ok(false) => {
            Self::deposit_event(Event::<T>::VerificationFailed);
            Ok(())
        },
        Err(_) => Err(Error::<T>::ProofVerificationError.into()),
    }
}
```

## Running
We are ready to build and run our Substrate node. We are going to run it in the development mode, where the chain doesn't require any peer connections to finalize blocks. Our pallet will be run inside its runtime. 
```
cargo run -- --dev
```
If everything goes fine, we should get the output:
```
2023-01-04 13:46:22 Substrate Node    
2023-01-04 13:46:22 ✌️  version 4.0.0-dev-91c730faef3    
2023-01-04 13:46:22 ❤️  by Substrate DevHub <https://github.com/substrate-developer-hub>, 2017-2023    
2023-01-04 13:46:22 📋 Chain specification: Development    
2023-01-04 13:46:22 🏷  Node name: tan-fold-8157    
2023-01-04 13:46:22 👤 Role: AUTHORITY    
2023-01-04 13:46:22 💾 Database: RocksDb at /var/folders/fj/55qm1f3s5pqcsts0z533kjbw0000gn/T/substrateXzNqQ6/chains/dev/db/full    
2023-01-04 13:46:22 ⛓  Native runtime: node-template-100 (node-template-1.tx1.au1)    
2023-01-04 13:46:32 🔨 Initializing Genesis block/state (state: 0x3d2e…2cf2, header-hash: 0x727c…b779)    
2023-01-04 13:46:32 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2023-01-04 13:46:41 Using default protocol ID "sup" because none is configured in the chain specs    
2023-01-04 13:46:41 🏷  Local node identity is: 12D3KooWDvUYJJvyfMx2D6bdPwF1DwTQ5xoPo1QNAhwCxdsFWSMW    
2023-01-04 13:46:41 💻 Operating system: macos    
2023-01-04 13:46:41 💻 CPU architecture: x86_64    
2023-01-04 13:46:41 📦 Highest known block at #0    
2023-01-04 13:46:42 〽️ Prometheus exporter started at 127.0.0.1:9615    
2023-01-04 13:46:42 Running JSON-RPC HTTP server: addr=127.0.0.1:9933, allowed origins=None    
2023-01-04 13:46:42 Running JSON-RPC WS server: addr=127.0.0.1:9944, allowed origins=None    
2023-01-04 13:46:42 Accepting new connection 1/100
2023-01-04 13:46:47 💤 Idle (0 peers), best: #0 (0x727c…b779), finalized #0 (0x727c…b779), ⬇ 0 ⬆ 0    
2023-01-04 13:46:48 🙌 Starting consensus session on top of parent 0x727cdb18c7e5b4007d76277358d70c63ecdef8770bd65d5b44c77a1d7afcb779    
2023-01-04 13:46:48 🎁 Prepared block for proposing at 1 (3 ms) [hash: 0x2e7a5acb2f1a0bda1cb5157d669da453faa9bd5235eb11d675fa9ace1659e388; parent_hash: 0x727c…b779; extrinsics (1): [0x9bc3…dbfd]]    
2023-01-04 13:46:48 🔖 Pre-sealed block for proposal at 1. Hash now 0x28c492e129b7c7a82b6c5d26891b40ba7462f97aa02a1f9131a819ca865da2eb, previously 0x2e7a5acb2f1a0bda1cb5157d669da453faa9bd5235eb11d675fa9ace1659e388.    
2023-01-04 13:46:48 ✨ Imported #1 (0x28c4…a2eb) 
```

Now we are ready to test our pallet. We can do it, by using [polkadotjs](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) app. If you haven't done this before, you will need first to change the configuration to point to your local node. 

This app will allow us to interact with our zk-SNARK pallet, but first you will need to navigate to `Extrinsics` panel (*Developer -> Extrinsics*).

<center>
    
![alt zk-snark extrinsics!](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/img/extrinsicse_tab.png "Extrinsics tab")
    
</center>
    
In the field `submit the following extrinsic`, please select `zkSnarks`. This is the pallet, that we created during this tutorial. Now you should be able to see our two extrinsic. We are going to select `setupVerification(pubInput, vecVk)` and upload public inputs and the verification key. Normally this would be done by Bob, so we will switch to the his account. We are going to upload a file, so we need to select `file upload` for `pubInput` and `vecVk` as on. Now you can upload them appropriately. For `pubInput` select a file `public.json` and for the `vecVk` chose `verification_key.json`.

<center>

![alt zk-snark setup verification key!](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/img/vk.png "Setup Verification")

</center>

To upload them on the blockchain, by clicking on the `Submit Transaction` button. We are finally able to help Alice with the proof verification. In the same panel, we need to choose our second extrinsic, which is `verify`. We will have to do the same, as we did with the previous files, but this time we will upload proof. Please remember to switch to the Alice account.

<center>

![alt zk-snark proof!](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/img/proof.png "Proof")

</center>

When we press on `Submit Transaction`, our proof is going to be uploaded and the verification procedure will be run.

<center>

![alt zk-snark verification success!](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/img/verification_success.png "Verification Success")

</center>


Now we can verify if we received a `VerificationSuccess` event. To do it, we need to switch to the `Explorer` panel.

<center>

![alt zk-snark verification event!](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post/blog/img/event.png "Verification Event")

</center>

As you see, verification succeed, and the event was emitted from the Alice account.

## Summary
Thanks to blockchain technology and zk-SNARKs, Alice proved that she knew the solution to the Bob puzzle without revealing it. Everything was stored on the blockchain, so the result of the contest was fully transparent for everyone.
