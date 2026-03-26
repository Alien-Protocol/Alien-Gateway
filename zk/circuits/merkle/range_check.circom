pragma circom 2.0.0;

include "circomlib/circuits/comparators.circom";
include "circomlib/circuits/bitify.circom";

// RangeCheck
// Proves `leaf_before < value < leaf_after` (strict bounds).
//
// We also decompose the inputs into `nBits` to ensure they are interpreted
// as numbers in [0, 2^nBits), avoiding wraparound issues in field arithmetic.
template RangeCheck(nBits) {
    signal input leaf_before;
    signal input value;
    signal input leaf_after;

    signal output inRange;

    // Constrain all three values to fit into nBits (enforces < 2^nBits).
    component beforeBits = Num2Bits(nBits);
    component valueBits = Num2Bits(nBits);
    component afterBits = Num2Bits(nBits);
    beforeBits.in <== leaf_before;
    valueBits.in <== value;
    afterBits.in <== leaf_after;

    // Strict comparisons: (leaf_before < value) AND (value < leaf_after)
    component ltBefore = LessThan(nBits);
    component ltAfter = LessThan(nBits);
    ltBefore.in[0] <== leaf_before;
    ltBefore.in[1] <== value;
    ltAfter.in[0] <== value;
    ltAfter.in[1] <== leaf_after;

    inRange <== ltBefore.out * ltAfter.out;
}

