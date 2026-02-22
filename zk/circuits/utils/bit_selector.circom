pragma circom 2.0.0;

// DualMux/BitSelector
// Takes two inputs and a selector bit
// If s == 0, returns [in[0], in[1]]
// If s == 1, returns [in[1], in[0]]
template BitSelector() {
    signal input in[2];
    signal input s;
    signal output out[2];

    s * (1 - s) === 0; // s must be 0 or 1

    out[0] <== (in[1] - in[0]) * s + in[0];
    out[1] <== (in[0] - in[1]) * s + in[1];
}
