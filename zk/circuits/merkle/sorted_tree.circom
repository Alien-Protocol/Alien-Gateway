pragma circom 2.0.0;

include "path_calculator.circom";
include "circomlib/circuits/bitify.circom";

// SortedTreeConsecutive
// Verifies:
// 1) leaf_before is included in the Merkle tree at the path given
// 2) leaf_after  is included in the Merkle tree at the path given
// 3) their leaf indices (derived from pathIndices) are consecutive
//
// The tree is assumed to store leaves in sorted order by value off-chain,
// so consecutive indices correspond to consecutive values.
template SortedTreeConsecutive(levels) {
    // Public inputs
    signal input root;

    // Boundary leaves (private)
    signal input leaf_before;
    signal input leaf_after;

    // Merkle paths for each boundary leaf
    signal input merklePathBeforeSiblings[levels];
    signal input merklePathBeforeIndices[levels];
    signal input merklePathAfterSiblings[levels];
    signal input merklePathAfterIndices[levels];

    // Output
    signal output boundariesValid;

    // Verify inclusion of leaf_before
    component beforeCalc = PathCalculator(levels);
    beforeCalc.leaf <== leaf_before;
    for (var i = 0; i < levels; i++) {
        beforeCalc.pathElements[i] <== merklePathBeforeSiblings[i];
        beforeCalc.pathIndices[i]  <== merklePathBeforeIndices[i];
    }
    beforeCalc.root === root;

    // Verify inclusion of leaf_after
    component afterCalc = PathCalculator(levels);
    afterCalc.leaf <== leaf_after;
    for (var i = 0; i < levels; i++) {
        afterCalc.pathElements[i] <== merklePathAfterSiblings[i];
        afterCalc.pathIndices[i]  <== merklePathAfterIndices[i];
    }
    afterCalc.root === root;

    // Enforce consecutive indices:
    // We interpret each leaf's `pathIndices[levels]` as a binary number with
    // pathIndices[0] being the least-significant bit (matches PathCalculator usage).
    component idxBefore = Bits2Num(levels);
    component idxAfter = Bits2Num(levels);
    for (var i = 0; i < levels; i++) {
        idxBefore.in[i] <== merklePathBeforeIndices[i];
        idxAfter.in[i]  <== merklePathAfterIndices[i];
    }
    idxAfter.out === idxBefore.out + 1;

    boundariesValid <== 1;
}

