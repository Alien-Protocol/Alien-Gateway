pragma circom 2.0.0;

include "circomlib/circuits/poseidon.circom";
include "./username_hash_impl.circom";

template UsernameMerkle(depth) {

    signal input username[32];
    signal input pathElements[depth];
    signal input pathIndices[depth];

    signal output root;

    component userHash = UsernameHash();
    for (var i = 0; i < 32; i++) {
        userHash.username[i] <== username[i];
    }

    signal currentHash[depth + 1];
    currentHash[0] <== userHash.username_hash;

    component hashers[depth];

    signal left[depth];
    signal right[depth];
    signal a[depth];
    signal b[depth];
    signal c[depth];
    signal d[depth];

    for (var i = 0; i < depth; i++) {

        pathIndices[i] * (pathIndices[i] - 1) === 0;

        hashers[i] = Poseidon(2);

        a[i] <== currentHash[i] * (1 - pathIndices[i]);
        b[i] <== pathElements[i] * pathIndices[i];
        left[i] <== a[i] + b[i];

        c[i] <== pathElements[i] * (1 - pathIndices[i]);
        d[i] <== currentHash[i] * pathIndices[i];
        right[i] <== c[i] + d[i];

        hashers[i].inputs[0] <== left[i];
        hashers[i].inputs[1] <== right[i];

        currentHash[i + 1] <== hashers[i].out;
    }

    root <== currentHash[depth];
}

component main = UsernameMerkle(2);