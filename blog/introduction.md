# Zk-SNARKs with Substrate (Part 1)

**In this article I would like to introduce you with the zk-SNARKs (zero-knowledge succinct non-interactive argument of knowledge) concept. First we are going to briefly describe what are the zero-knowledge proofs, what are the stages of creating them, which tools can be useful for generating the zk-SNARKs. Also, we will touch a little math behind them. I encourage you to visit our GitHub, where you can find a [repository](https://github.com/bright/zk-snarks-with-substrate) for this article. Let’s start with the definition of the zero-knowledge proof and then we will move to the zk-SNARKs.**

Zero-knowledge proof is a method where one party (the prover) tries to “*convince*” the other party (the verifier) that a given statement is true, without revealing the solution. There are two types of proving systems:
* **interactive** - where the prover and verifier exchange multiple messages with each other, until the verifier is convinced enough that the prover knows the given statement is true.
* **non-interactive** - where the prover creates a proof and the verifier is able to run it and check if the given statement is true. Compared to the interactive version, there is only one message (proof) that is sent to the verifier and it can be run asynchronously.

**In this article we will focus on the non-interactive approach which is called zk-SNARKs (zero-knowledge succinct non-interactive argument of knowledge).**

## zk-SNARKs

From the high-level point of view, the concept defines:
* $y$ - public inputs that are known to everyone.
* $x$ - private inputs that are only known for the prover. He claims that these are the right inputs for solving the problem.
* **Problem** - a function $f(x,y)$, which takes private and public inputs. Result of this function is boolean: $true$ or $false$.
* **Prover** - he knows the solution for the problem (private inputs), based on that he can create a proof.
* **Verifier** - he can accept a proof and verify it.

<center>
    
![alt zk-snark concept!](https://i.imgur.com/8H5rSW2.png "Concept diagram")

</center>

As shown on the image, the prover will create a proof, based on the public and private inputs. Verifier will receive it and run the verification knowing only the public inputs. Based on that we can conclude that proof will need somehow to wrap our problem and the private inputs. Then it will need to transform them to the other form which could be verified only with the public inputs. Now when we know the concept, we can dive deeper and look at this process in detail.

First let’s think about the problems which zk-SNARKs can solve. There are two types of problems P(*deterministic polynomial*) and NP (*nondeterministic polynomial*). The first ones are the problems that can be run in polynomial time and those are not applicable for the zk-SNARKs. The second ones are the problems, which can only be verified in polynomial time. In other words, finding the right solution for an NP problem is very hard, but verifying an already existing one is quite easy. This is exactly what zk-SNARKs is about, but first our problem will need to be transformed to proper form, which is a QAP (Quadratic Arithmetic Program). This is actually a process where we transform a code into a mathematical representation of it.

If we want to go further, we will need to have a suitable example, which helps us in better understanding the concept of zk-SNARKs. **Let’s assume that Bob is a founder of the Bright Coders union**. In front of his mates, he announces that there are few places left in the union and **those who solve this equation first:

<center>

$$ x^2+3=12 $$

</center>

will be able to join it!** Alice is one of his friends who knows the result, which is $x=3$. Instead of saying it loudly and risking others to claim the vacancy, she is willing to use a zk-SNARKs to prove to Bob that she knows the result without revealing it!

If we take a closer look at the equation and set it together with what we already knew about the zk-Snarks, we will notice two things:
* Equation is well know to everyone, so result “*12*” can be our “*public input*”
* “*x*” is what we are looking for, so this can be our “*private input*”. This matches the requirement for only a prover (Alice) to know its value.

Now together with Alice we will try to explain the process of converting the equation above into the zk-SNARK. The process takes a couple of stages:
* Computation statement
* Flattening
* R1CS
* QAP

We are going to describe them in the next part of this article. Alice is going to use two external tools [Cricom](https://docs.circom.io/getting-started/installation/) and [SnarkJS](https://github.com/iden3/snarkjs). Circom is a compiler written in Rust for creating circuits. SnarkJS is a npm package, which implements generation and validation of the zk-SNAKRs for the artifacts produced by Circom. For the installation process, please check our [documentation](https://github.com/bright/zk-snarks-with-substrate/blob/main/circom/README.md) in the repository.


## Computation statement
Alice will start with writing our equation as a Rust function
```
fn solution(x: i32) -> i32 {
   let y = x*x + 3;
   return y;
}
```
by using it, she can easily verify if value "*3*" is the right answer for our equation. At this stage, it is good to point out that Alice could build a binary and send it to Bob asking him to verify the result. Unfortunately we have two problems here:
* Alice will need to reveal the value of the “*x*” variable to Bob, which she doesn’t want to.
* Bob will not be sure if Alice's program is the correct one. For example her program could just return “*12*”, without doing any computations.

Solving those problems can be done by converting this program to QAP (*Quadratic Arithmetic Program*) and adding some cryptography. This is what we are going to do next steps.

## Flattening

We need to convert our computation statement to a few smaller ones which have one of two forms:

* Assignment to the variable or constants ($x=y$, where “*y*” be a variable or a constant)
* Assignment to the combination of operators ($x=y (op) z$ , where “*op” is one of the $(+,-,*,/)$ and “*y*” and “*z*” can be variables or constants).

You can think about those statements as gates of an arithmetic circuit. The result of the flattening for our statement will be:

```
fn solution(x: i32) -> i32 {
   let tmp1 = x*x;
   let y = tmp1 + 3;
   return y;
}
```

## Rank-1 Constraint System

Next step is to convert our circuits to a R1CS (*rank-1 constraint system*), which is a list of three vectors $a,b,c$ and a solution to R1CS which is a vector $s$, such that:

<center>

$$a_{i}\cdot s * b_{i}\cdot s - c_{i}\cdot s = 0$$

</center>

where:

* “ $\cdot$ ” is a dot product
* $i$ in $[1,N]$ and $N$ is a number of circuits

We can interpret this in this way, if our vectors could represent the constraints (equation which describes circuits), then vector $s$ will be our witness, that satisfies the equation above. 

We will start with the definition of $s$, which is a vector of all values associated with the variables that have been used in the circuits. Based on this our vector $s$ will represent a variable in such a way $1,y,x,tmp1$. First element is always $1$ and will represent constant values. Then are the public inputs $y$, private inputs $x$ and other intermediate variables $tmp1$. If we replace variables with the values, we will get our vector, which we call a witness:

<center>

$$ s=[1,12,3,9] $$

</center>

Now when we defined the witness we can conclude the vectors for the $a,b,c$, which will actually map to our circuits. Please have in mind that our vector $s$ maps to $[1,y,x,tmp1]$, based on that vectors $a,b,c$ for the:

first circuit $(tmp1=x*x)$:

<center>

$$a_{1}=[0,0,1,0]$$
$$b_{1}=[0,0,1,0]$$
$$c_{1}=[0,0,0,1]$$

</center>


second circuit $(y=tmp1+3)$:

<center>

$$a_{2}=[0,0,0,0]$$
$$b_{2}=[0,0,0,0]$$
$$c_{2}=[3,1,0,1]$$

</center>

If we put everything together for the first circuit, we can check the correctness of the R1CS equation:

<center>

| $a_{1}$ | $b_{1}$ | $c_{1}$ | $s$ |
| :-: | :-: | :-: |:-: |
| 0 | 0 | 0 | 1|
| 0 | 0 | 0 | 12|
| 1 | 1 | 0 | 3|
| 0 | 0 | 1 | 9|

</center>

<center>

$$a_{i}\cdot s * b_{i}\cdot s - c_{i}\cdot s = 3*3 -9 = 0$$

</center>

As you can see the computations are fine, so R1CS for circuit one is ok. Alice will now use a *Cricom* for generating a R1CS. First she need to creates a *Cricom* template file (*task.cricom*) which defines a constraints for our code:

```
pragma circom 2.0.0;
template Task() {
   signal input x;
   signal output y;
   signal tmp_1;
 
   tmp_1 <== x * x;
   y <== tmp_1 + 3;
}
component main = Task();
```

Than she can generate R1CS by running the command:

`circom task.circom --r1cs --wasm --sym --c --o build --O0 --p bls12381`

This will generate a file (*task.r1cs*) which describes a R1CS in the *Cricom*. After that Alice will need to create a witness (vector $s$) file, but this time she will use a *SnarkJS* tool for doing this. First she needs to create a json file which will describe all private inputs in our circuits. In our case this is a very simple task, because our only input value is $x$ which is “*3*”. Input file (*input.json*) will look like this:

```
{"x": "3"}
```

Than she can generate a witness:
`node generate_witness.js task.wasm ../../input.json witness.wtns`

Alice can also verify the result, by exporting witness to json:

`snarkjs wtns export json witness.wtns witness.json`

```
[
 "1",
 "12",
 "3",
 "9"
]
```

As you can see, the result is exactly the same as it were for our witness from the computations.

## Quadratic Arithmetic Program

The last step is to convert a R1CS to QAP, which will allow us to transform R1CS vectors to the polynomials. The logic behind the equation will still be the same, but instead of using vectors with a dot product we will use polynomials. We can start with the declaration of the polynomials $A_{i}(x)$, $B_{i}(x)$ and $C_{i}(x)$ for $i$ in $[1,N]$, where the $N$ is a number of variables for our constraints (in our case it will be 4). Than we can create a set of points for $A_{i}(n)=a_{n}(i)$ and similar for $B_{i}(n)$ and $C_{i}(n)$. Based on those points, we can create polynomials by using a [Lagrange interpolation](https://en.wikipedia.org/wiki/Lagrange_polynomial). As a result we will get a set of polynomials which can be then written in the equation:

<center>

$$ A(X)*B(X)-C(X)=H(X)*Z(X) $$

</center>


where:

$Z(X)=(x-x_{1})*(x-x_{2})...(x-x_{n})$, where $n$ is number of constraints

$H(X)$ - is some polynomial which we define further

Why can we write this equality here? As you remember we are now in the scope of the polynomials, so we can define a polynomial $P(X)$, which will be:

<center>

$$ P(X)=A(X)*B(X)-C(X)=0 $$

</center>

From the [polynomial long division](https://en.wikipedia.org/wiki/Polynomial_long_division), we can deduce that above equation will only hold, if $P(X)$ will be divided by the $Z(X)=(x-x_{1})*(x-x_{2})...(x-x_{n})$ without a reminder. Our formula can be written like this:

<center>

$$\frac{P(X)}{Z(X)}=\frac{A(X)*B(X)-C(X)}{(x-1)*(x-1)}=H(X) $$

</center>

Ok, so we get the definition of $H(X)$, but what do we actually gain by conversion from R1CS to QAP? It appears that previously in R1CS to verify the equation we need to verify dot product calculation for each constraint separately, now using polynomials we can verify all constraints at once. I will just remind you, the prover will try to convince the verifier that the equation $A(X)*B(X)-C(X)=H(X)*Z(X)$ holds.

Alice knows the witness and she’s able to compute $H(X)$. By expressing computations with the R1CS, Bob will be certain that Alice is running the right computations and she is not cheating. If only Alice could somehow hide the information about the witness from Bob. The best way to do it will be using some cryptography.

## Summary

At this point we are going to stop. What we already learned is what the zk-SNARKs are and how we can use tools like *Circom* and *SnarkJS* in creating them. In the next post, we will take a closer look at the *Groth16*, which is a cryptography proof system that will allow us to finish the Alice task. For more information I encourage you to check our links.

### Links

* https://medium.com/@VitalikButerin/quadratic-arithmetic-programs-from-zero-to-hero-f6d558cea649
* https://blog.decentriq.com/zk-snarks-primer-part-one/
* https://vitalik.ca/general/2021/01/26/snarks.html
* https://xord.com/research/explaining-quadratic-arithmetic-programs/
* https://www.zeroknowledgeblog.com/index.php/zk-snarks
