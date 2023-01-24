# Zk-SNARKs in Substrate (Part 2). Using Groth16 and Running Circom Proof

As you might remember from the first blog post of our “Zk-snarks with Substrate” series we learned about zk-SNARKs. Also, we explained how we can use tools like Circom and SnarkJS in creating zk-SNARKs. If you haven't checked that article, you should do that. 

Now it’s high time for Alice to prove that she knows the result of Bob’s equation 

<center>

$$ x^2+3=12 $$

</center>

and is worthy of joining his Bright Coders union. ;) Alice will use the Groth16 proof system to achieve that.

## Introduction to Groth16

Groth16 is a cryptography proof system proposed by Daniel Jens Groth in the paper “[On the Size of Pairing-based Non-interactive Arguments](https://eprint.iacr.org/2016/260.pdf)” published in 2016. It’s a zero-knowledge proof protocol, and one of the most popular zkSNARK proving schemes.

Here are the three core properties of Groth16 (according to Groth’s paper):
* **Completeness**: Given a statement and a witness, the prover can convince the verifier.
* **Soundness**: A malicious prover cannot convince the verifier of a false statement.
* **Zero-knowledge**: The proof does not reveal anything but the truth of the statement,in particular, it does not reveal the prover’s witness.

## Groth16 Overview
Our previous article ended, when we defined a Quadratic Arithmetic Program (QAP):
<center>

$$ A(x)*B(x)-C(x)=H(x)*Z(x) $$

</center>


We made there a statement that the prover will try to convince the verifier that the above equation holds. Now we will take a closer look at how this statement applies to Groth16. Like other QAP-based algorithms, the prover will need somehow to prove that he knows a witness ($w$). As you remember from the previous article, a witness is a vector that contains all public and private inputs. In other words, the witness holds the complete solution for the Rank-1 Constraint System (R1CS), and in consequence, is a solution for our problem. We need to remember that $A(x)$, $B(x)$ and $C(x)$ are actually:
<center>

$A(x)=\sum_{i=1}^{m} w_i*A_i(x)$
$B(x)=\sum_{i=1}^{m} w_i*B_i(x)$
$C(x)=\sum_{i=1}^{m} w_i*C_i(x)$

</center>

where:
$w$ - witness
$A_i,B_i,C_i$ - are the polynomials

In the previous article, we made another statement that Groth16 is a cryptographic proving system, which is used to encrypt information about the witness. In other words, the verifier will never know the solution to their problem. Groth16 achieves this by two paring friendly curves $G1$, $G2$ (with the paring domain $Gt$)[^2].

As we mentioned earlier, Groth16 requires a trusted setup. This process creates a *Common Reference String* (CRS), which are encrypted secrets that are required by the prover and the verifier. To be more precise, during this process we are generating the following elements: $\alpha$, $\beta$, $\gamma$, $\delta$, $\tau$, which are our toxic waste[^3].

Groth16 also defines a polynomial:
<center>

$$ L_i(x)= \beta*A_i(x)+\alpha*B_i(x)+C_i(x)$$
    
</center>
which is going to be used in further computations.

The result of the trusted setup are generated keys for the prover and the verifier. They are going to be used in the process of creating proof and its verification.

Now let's move to the proof of its own. The prover is going to create it, based on the witness knowledge and the prover key. He is going to construct three values: $A_p$, $B_p$ and $C_p$, which are:
<center>

$A_p=\alpha+A(\tau)+r*\delta$, where $A(\tau)=\sum w_i*A_i(\tau)$ for $i$ in $0..m$

</center>
    
<center>

$B_p=\beta+B(\tau)+s*\delta$, where $B(\tau)=\sum w_i*B_i(\tau)$ for $i$ in $0..m$

</center>

<center>
    
$C_p=Laux(\tau)/\delta+H(\tau)*Z(\tau)/\delta+s*A_p+r*B_p-r*s*\delta$, 
    where $Laux(\tau)=\sum w_i*L_i(\tau)$ for $i$ in $l+1..m$
    
</center>

$r$ and $s$ are randomly generated field elements. This is our proof and the  $Laux(\tau)$ is a calculation only for the private inputs.

Now we move to the verifier. The goal for him is to check if the QAP holds when we evaluate it with the random $\tau$ value generated during the trusted setup:

<center>
    
$$ A(\tau)*B(\tau)-C(\tau)=H(\tau)*Z(\tau) $$

</center>

Unfortunately, the verifier doesn't have all pieces to make such a computation. Instead, he received proof from the prover, which contains only values for $A_p$, $B_p$, and $C_p$. We can transform the above QAP formula into this form:

<center>
    
$$ A_p*B_p=\alpha*\beta+(\sum w_i*L_i(\tau)/\gamma)*\gamma+Cp*\delta $$

</center>

where $i$ is for $0..l$ (all public inputs).

How did this happen? If you replace the values of the above equation with the proof equations you will end up with this[^2]:

<center>
    
$$ A(\tau)*B(\tau)+REM= C(\tau)+H(\tau)*Z(\tau)+REM$$

</center>

where: 
$REM=\alpha*\beta+\alpha*B(\tau)+\beta*A(\tau)+\alpha*s*\delta+s*\delta*A(\tau)+r*\delta*B(\tau)+r*\beta*\delta+r*s*\delta*\delta$
As you can see, after $REM$ reduction, we will get the same QAP equation evaluated in $\tau$, which was our goal.

## Creating a proof – step by step

We are ready now to create a proof. In our previous article, we started the process of creating it using Circom and SnarkJS. We wrote a circuit template and compiled it into the R1CS (Rank-1 Constraint System) form. We created a witness for R1CS, based on Alice's knowledge of solving Bob's equation. Now it is time to finish this process and finally create a proof.

Let's remind us what artifacts we had already created:
* **[input.json](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post-groth/blog/data/input.json)** - this file contains public input, in our case, it is a `12` value from our equation.
* **[task.r1cs](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post-groth/blog/data/task.r1cs)** - constrains in R1CS format.
* **[task.wasm](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post-groth/blog/data/task.wasm)** - circuit compiled to WebAssembly.
* **[witness.wtns](https://github.com/bright/zk-snarks-with-substrate/blob/M2-post-groth/blog/data/witness.wtns)** - witness file

We will start now by creating a trusted setup for the Groth16. This will create a shared set of values and the process can be split into two parts. First, generic for all proofs, and the second one specific to the circuit.

### Power of Tau

The first command starts the power of the tau ceremony. We are declaring here which curve are we going to use, in our case it is **bls12381**. Then we are setting the maximum number of constraints that can be accepted in the ceremony. We set it to *12*, but this is the exponent value of 2, so the number of constraints is $2^{12}=4096$. Finally, we are setting the output file *pot12_0000.ptau*. 
```
snarkjs powersoftau new bls12381 12 pot12_0000.ptau -v
```

The second command is a contribution to the ceremony. It will create a new *pot12_0001.ptau* file, which is a transcription of the old one. We are adding text to provide an extra source of entropy.
```
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="ZkSnarks phase #1" -v
```

### Phase 2
The second phase is specific for the circuit. Under the hood of this command, it will calculate the encrypted evaluation of the Lagrange polynomials at tau for $tau$, $alpha*tau$ and $beta*tau$[^1]. 
```
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
```

Next, we will start setting up Groth16. As we mentioned at the beginning of this article, Groth16 requires a trusted step. By running this command, we are generating a zero-knowledge key (*zkey*). It contains proving and the verification key as well as phase 2 contributions.
```
snarkjs groth16 setup task.r1cs pot12_final.ptau task_0000.zkey
```

Now similar to *power of tau* step, we are going to contribute to the phase 2 ceremony. Once again we are adding text to provide an extra source of entropy.
```
snarkjs zkey contribute task_0000.zkey task_0001.zkey --name="ZkSnarks phase #2" -v
```

### Generate a proof
Finally, we are able to create a proof for Alice. This command will generate a *proof.json* and the *input.json* files. The first one, contains the actual proof, while the second is the values of the public inputs and outputs.
```
snarkjs groth16 prove task_0001.zkey task_js/witness.wtns proof.json input.json
```

## Verifying a proof
Alice is ready to show her proof to Bob and win the contest. Before doing this, Bob will need to extract a verification key from the zero-knowlege keys. He is going to use it to verify if the proof is valid. In other words, he will check if proof provided by Alice corresponds to the right circuit and in consequence to Bobs math puzzle.

A command bellow will export the verification key from the zero-knowlege keys:
```
snarkjs zkey export verificationkey task_0001.zkey verification_key.json -v
```

Finally, Bob can verify Alice's proof with the Groth16:
```
snarkjs groth16 verify verification_key.json input.json proof.json
```

and the result is:
```
[INFO]  snarkJS: OK!
```
which means that proof passes the verification.


## Running Circom proof in Rust

In the last chapter of this article, we are going to present our Rust implementation of the Groth16 (verification part). There are Rust libraries like [Bellman](https://crates.io/crates/bellman), which can be used to implement circuits or even run a full Groth16 process, but they are not compilable to WebAssembly (*WASM*). 

Our next article will focus on running a zk-SNARK verification process on the blockchain, and we are going to use a Substrate for this. The main condition for building a blockchain with the Substrate is the capability of compiling code to the WASM, which is added to the runtime of the Substrate node. This was the reason why we decided to implement our version of Groth16 verification process, which compiles to WASM.

You can find our code on [Github](https://github.com/bright/zk-snarks-with-substrate).

We need to start with the definition of our goals. We would like to run a Groth16 verification process of the artifacts generated by the Circom and SnarkJS. In other words, we would like to do the same as Bob did when he verified Alice's proof. Our code is going to be a part of the Substrate pallet, which is built into the runtime of the Substrate node.

When we defined our goals, we can focus now on the verification part. At the beginning of this article, we presented what it consists of Groth16 proof and how the verifier can verify it. The core formula for us is:

<center>
    
$$ A_p*B_p=\alpha*\beta+(\sum w_i*L_i(\tau)/\gamma)*\gamma+Cp*\delta $$

</center>

where, 
$A_p,B_p,C_p$ - values defined in the proof
$\alpha,\beta,\gamma,\tau,\delta$ - values created during the trusted setup
$L_i(\tau)$ - value of the polynomial defined in the Gorth16 evaluated for $\tau$
$w_i$ - is our witness
$i$ - is for $0..l$ (all public inputs).

Basically what we will need to verify, is to check if the left-hand side (LHS) is equal to the right-hand side (RHS) of this equation using data from the proof, inputs, and the verification key. Now let's see how this applies to our code.


### Verifying proof
Implementation is done in the Substrate node, where we are adding a new pallet called *zk-snarks*. Code for this pallet can be found in [lib.rs](https://github.com/bright/zk-snarks-with-substrate/blob/main/pallets/zk-snarks/src/lib.rs) file. We are going to discuss the pallet itself in the next article, now we will take a closer look at the verification function from the [verify.rs](https://github.com/bright/zk-snarks-with-substrate/blob/main/pallets/zk-snarks/src/verify.rs) file.

We can find there a function:
```
pub fn verify(vk: VerificationKey, proof: Proof, inputs: PublicInputs) -> VerificationResult{
...
}
```
which accepts a verification key, proof, and public inputs. It returns a verification result, which is defined as:
```
pub type VerificationResult = Result<bool, VerificationError>;
```
Our result is a boolean type, true if the verification succeeds and false if it fails. If anything goes wrong, we can return an error. The function implements our formula:

<center>
    
$$ A_p*B_p=\alpha*\beta+(\sum w_i*L_i(\tau)/\gamma)*\gamma+Cp*\delta $$

</center>

First, let's start from the RHS of our equation, and calculate a $\sum w_i*L_i(\tau)$, only for the $i$ in $0..l$, so we are only summing the public inputs. Please note that sum for the private inputs is hidden in our $C_p$. To calculate the final sum we need to multiply each `ic` element by $w_i$ (The first public input is always 1 so we can skip the first value).

```
let mut acc = vk.ic[0].to_curve();
for (i, b) in public_inputs.iter().zip(vk.ic.iter().skip(1)) {
    AddAssign::<&<Bls12 as Engine>::G1>::add_assign(&mut acc, &(*b * i));
}
```
Once we have all ingredients for pairing, we can compute it in two steps: multi miller loop and final exponentiation:
```
let final_result = Bls12::multi_miller_loop(&[
    (&vk.alpha, &vk.beta.into()),
    (&acc.to_affine(), &vk.gamma.into()),
    (&proof.c, &vk.delta.into()),
])
.final_exponentiation();
```
The result of `multi_miller_loop` function call allows us to perform the final exponentiation on it. So in the end we get `Gt` element which can be compared with the result of LHS calculated before.

The result of computation for our RHS `Gt` element which can be compared with the result of LHS.

Now, we move to the LHS, and we are calculating a paring for $A_p*B_p$
```
let a_b_pairing = Bls12::pairing(&proof.a, &proof.b);
```

At the end we do a final check to ensure the correctness of the proof:

`  Ok(a_b_pairing == final_result)`

### Running the verification

Finally, we can test if our Rust code works correctly with the Circom and SnarkJS artifacts. By typing the command:

```
cargo run --release -- zk-snarks-verify circom/build/verification_key.json circom/build/proof.json circom/build/input.json
```

we should get the output:
```
Proof OK
```

which means that verification succeeded.

## Ready for more?

Thanks to Circom proof based on Groth16 Alice was able to prove to Bob that she knows how to solve the equation. For now, we assumed that both of them are in the same place. But what if they weren’t? Alice can present her case in blockchain! You’ll learn how to show the solution with Substrate in the third and final part of our Zk-Snarks blog post series!


[^1]: https://github.com/iden3/snarkjs
[^2]: https://hongchao.me/zksnark/
[^3]: https://www.zeroknowledgeblog.com/index.php/groth16