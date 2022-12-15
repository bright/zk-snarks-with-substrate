pragma circom 2.0.0;
template Task() {
    signal input x;
    signal output y;
    signal tmp_1;

    tmp_1 <==  x * x;
    y <== tmp_1 + 3;
 }
component main = Task();
