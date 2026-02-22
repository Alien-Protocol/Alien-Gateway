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
