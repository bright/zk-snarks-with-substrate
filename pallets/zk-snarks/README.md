# ZK-Snarks pallet

Building the project
```
cargo build --manifest-path=../../Cargo.toml --release
```

Running unit tests:
```
cargo test
```

Running benchmark tests:
```
cargo test --package pallet-zk-snarks --features runtime-benchmarks
```

Benchmarking:
```
cargo build --manifest-path=../../Cargo.toml --package node-template --release --features runtime-benchmarks
```

Generating weights:
```
../../target/release/node-template benchmark pallet \
--chain dev \
--pallet pallet-zk-snarks \
--extrinsic '*' \
--steps 20 \
--repeat 10 \
--output src/weights.rs
```