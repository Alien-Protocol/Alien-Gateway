pragma circom 2.0.0;
include "poseidon_hasher.circom";
include "../utils/bit_selector.circom";

// Calculates the root from a leaf using a Merkle path
template PathCalculator(levels) {
    signal input leaf;
    signal input pathElements[levels];
    signal input pathIndices[levels];
    
    signal output root;

    component selectors[levels];
    component hashers[levels];

    signal current[levels + 1];
    current[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        selectors[i] = BitSelector();
        selectors[i].in[0] <== current[i];
        selectors[i].in[1] <== pathElements[i];
        selectors[i].s <== pathIndices[i];

        hashers[i] = PoseidonHasher();
        hashers[i].left <== selectors[i].out[0];
        hashers[i].right <== selectors[i].out[1];

        current[i + 1] <== hashers[i].hash;
    }

    root <== current[levels];
}
