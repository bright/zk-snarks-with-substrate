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

## Create a proof


#### Powers of Tau
This part is circuit independent.
```
snarkjs powersoftau new bls12381 12 pot12_0000.ptau -v
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v
```

#### Phase 2
```
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
snarkjs groth16 setup ../task.r1cs pot12_final.ptau task_0000.zkey
snarkjs zkey contribute task_0000.zkey task_0001.zkey --name="1st Contributor Name" -v
snarkjs zkey export verificationkey task_0001.zkey verification_key.json
```

#### Generating a Proof
```
snarkjs groth16 prove task_0001.zkey witness.wtns proof.json public.json
```

## Verifying a Proof
```
snarkjs groth16 verify verification_key.json public.json proof.json
```