
pragma circom 2.0.0;

include "circomlib/circuits/poseidon.circom";

template UsernameHash() {

    // 32-character fixed username
    signal input username[32];

    // Public output
    signal output username_hash;

    // Step 1: Hash in chunks of 4
    component h[8];

    for (var i = 0; i < 8; i++) {
        h[i] = Poseidon(4);

        for (var j = 0; j < 4; j++) {
            h[i].inputs[j] <== username[i*4 + j];
        }
    }

    // Step 2: Hash intermediate hashes
    component h2[2];

    for (var i = 0; i < 2; i++) {
        h2[i] = Poseidon(4);
        h2[i].inputs[0] <== h[i*4].out;
        h2[i].inputs[1] <== h[i*4 + 1].out;
        h2[i].inputs[2] <== h[i*4 + 2].out;
        h2[i].inputs[3] <== h[i*4 + 3].out;
    }

    // Final hash
    component finalHash = Poseidon(2);
    finalHash.inputs[0] <== h2[0].out;
    finalHash.inputs[1] <== h2[1].out;

    username_hash <== finalHash.out;
}

component main = UsernameHash();

pragma circom 2.0.0;
include "../node_modules/circomlib/circuits/poseidon.circom";

// Hashes a username using Poseidon
template UsernameHasher() {
    signal input username;
    signal output hash;

    component poseidon = Poseidon(1);
    poseidon.inputs[0] <== username;
    hash <== poseidon.out;
}

