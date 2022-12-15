## Requirements
* Rust
* Circom
* SnarkJS

for a details of the installation process please check: https://docs.circom.io/getting-started/installation

## Building

### Compile the circuit
Create an R1CS for the circom template, output can be found in "*build*" directory.
```
circom task.circom --r1cs --wasm --sym -o build --O0 -p bls12381
```

## Computing the witness
We are going to use JavaScript way of building witness, byr running command:
```
cd build/task_js
node generate_witness.js task.wasm ../../input.json witness.wtns
```
will generate a `witness.wtns` file. We can now verify how our witness look like:

```
snarkjs wtns export json witness.wtns witness.json
```