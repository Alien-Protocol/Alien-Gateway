pragma circom 2.0.0;
include "path_calculator.circom";

// Verifies a Merkle path for a given root and leaf
template MerklePathVerifier(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component calculator = PathCalculator(levels);
    calculator.leaf <== leaf;
    for (var i = 0; i < levels; i++) {
        calculator.pathElements[i] <== pathElements[i];
        calculator.pathIndices[i] <== pathIndices[i];
    }

    root === calculator.root;
}
