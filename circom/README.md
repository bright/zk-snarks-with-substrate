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

## Powers of tau
Go back to our root
`cd ..`
Create ceremony
```
% snarkjs powersoftau new bls12381 12 pot12_0000.ptau -v
[DEBUG] snarkJS: Calculating First Challenge Hash
[DEBUG] snarkJS: Calculate Initial Hash: tauG1
[DEBUG] snarkJS: Calculate Initial Hash: tauG2
[DEBUG] snarkJS: Calculate Initial Hash: alphaTauG1
[DEBUG] snarkJS: Calculate Initial Hash: betaTauG1
[DEBUG] snarkJS: Blank Contribution Hash:
		786a02f7 42015903 c6c6fd85 2552d272
		912f4740 e1584761 8a86e217 f71f5419
		d25e1031 afee5853 13896444 934eb04b
		903a685b 1448b755 d56f701a fe9be2ce
[INFO]  snarkJS: First Contribution Hash:
		37093325 ac327a60 b4b95121 12b96771
		b8d17be1 36dcf0b4 d61ece67 414df8f0
		f1932706 478094b9 df867d05 86acd3d2
		d13a685b 8fe1d785 06f5cd61 f8982e2a
```
Contribute to ceremony
```
% snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="ZkSnarks phase #1" -v
Enter a random text. (Entropy): dsfdsfs
[DEBUG] snarkJS: Calculating First Challenge Hash
[DEBUG] snarkJS: Calculate Initial Hash: tauG1
[DEBUG] snarkJS: Calculate Initial Hash: tauG2
[DEBUG] snarkJS: Calculate Initial Hash: alphaTauG1
[DEBUG] snarkJS: Calculate Initial Hash: betaTauG1
[DEBUG] snarkJS: processing: tauG1: 0/8191
[DEBUG] snarkJS: processing: tauG2: 0/4096
[DEBUG] snarkJS: processing: alphaTauG1: 0/4096
[DEBUG] snarkJS: processing: betaTauG1: 0/4096
[DEBUG] snarkJS: processing: betaTauG2: 0/1
[INFO]  snarkJS: Contribution Response Hash imported:
33baac10 63e0d717 386a6802 dfc2dcfc
be27f36a f0aa848b 035657a3 44761e7a
7e000235 47e6ebf9 f3d1c4ac 858e6fce
73adf651 04bf3311 44424ffe 28ec43ef
[INFO]  snarkJS: Next Challenge Hash:
1c847ad8 9471b93a 7e746080 5c386e8c
82a1ba25 4a041721 d9b5ec53 7d68b836
b818e844 6b609dc4 e03849bd 9c98ef23
7386cbd2 d3d55f93 f36b4087 7103fd2d
```


Start phase 2 (outputs a lot)
```
% snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
[DEBUG] snarkJS: Starting section: tauG1
[DEBUG] snarkJS: tauG1: fft 0 mix start: 0/1
[DEBUG] snarkJS: tauG1: fft 0 mix end: 0/1
[DEBUG] snarkJS: tauG1: fft 1 mix start: 0/1
[DEBUG] snarkJS: tauG1: fft 1 mix end: 0/1
[DEBUG] snarkJS: tauG1: fft 2 mix start: 0/1
[DEBUG] snarkJS: tauG1: fft 2 mix end: 0/1
[DEBUG] snarkJS: tauG1: fft 3 mix start: 0/1
[DEBUG] snarkJS: tauG1: fft 3 mix end: 0/1
[DEBUG] snarkJS: tauG1: fft 4 mix start: 0/2
[DEBUG] snarkJS: tauG1: fft 4 mix start: 1/2
[DEBUG] snarkJS: tauG1: fft 4 mix end: 0/2
[DEBUG] snarkJS: tauG1: fft 4 mix end: 1/2
[DEBUG] snarkJS: tauG1: fft  4  join: 4/4
[DEBUG] snarkJS: tauG1: fft 4 join  4/4  1/1 0/1
...
[DEBUG] snarkJS: betaTauG1: fft  12  join: 12/12
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 7/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 0/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 1/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 4/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 3/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 2/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 6/8
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 5/8
```

Generate zkey:
```
% snarkjs groth16 setup build/task.r1cs pot12_final.ptau task_0000.zkey

[INFO]  snarkJS: Reading r1cs
[INFO]  snarkJS: Reading tauG1
[INFO]  snarkJS: Reading tauG2
[INFO]  snarkJS: Reading alphatauG1
[INFO]  snarkJS: Reading betatauG1
[INFO]  snarkJS: Circuit hash:
		8ddfc7a5 ed4b4047 016ba31b e9209338
		e2cc0004 4161cc44 815c6193 2bf839f6
		fe433964 9a5955a1 5d4e2bba 427ec1ff
		6138e9ad ab59158c 5dd2d077 8793ecd9
```

Contribute to phase 2
```
% snarkjs zkey contribute task_0000.zkey task_0001.zkey --name="ZkSnarks phase #2" -v
Enter a random text. (Entropy): random
[DEBUG] snarkJS: Applying key: L Section: 0/2
[DEBUG] snarkJS: Applying key: H Section: 0/4
[INFO]  snarkJS: Circuit Hash:
		8ddfc7a5 ed4b4047 016ba31b e9209338
		e2cc0004 4161cc44 815c6193 2bf839f6
		fe433964 9a5955a1 5d4e2bba 427ec1ff
		6138e9ad ab59158c 5dd2d077 8793ecd9
[INFO]  snarkJS: Contribution Hash:
		3897ddbb a9129eb7 ed0f2f2a b85d9e6d
		c33c386c 0e3f157a 9f717ddf b3667806
		f67dd4d0 dd47ae04 ea7ecd15 00240093
		011a6fe4 344b416f 82996809 0e0a04e0
```

Export the verification key:
`snarkjs zkey export verificationkey task_0001.zkey verification_key.json -v`


## Generating proof
`snarkjs groth16 prove task_0001.zkey build/task_js/witness.wtns proof.json input.json`

## Verifying proof
```
% snarkjs groth16 verify verification_key.json input.json proof.json
[INFO]  snarkJS: OK!
```

## Predefined commands

Alternatively you can run predefined commands to simplify process described above.
Commands are defined in `justfile`
In order to be able to run them you need to install `just` command runner, see https://github.com/casey/just#installation

For example to build circuits just type `just build`

In order to perform full tau ceremony just type `just tau`

In order to generate proof just type `just generate-proof`

In order to verify proof just type `just verify-proof`
