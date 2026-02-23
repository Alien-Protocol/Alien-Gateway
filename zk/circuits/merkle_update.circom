pragma circom 2.0.0;

include "circomlib/circuits/poseidon.circom";

template MerkleUpdate(depth) {

    // PRIVATE
    signal input oldLeaf;
    signal input newLeaf;
    signal input pathElements[depth];
    signal input pathIndices[depth];

    // PUBLIC
    signal input oldRoot;
    signal input newRoot;

    signal output isValid;

    // Enforce empty slot
    oldLeaf === 0;

    // ---------- OLD ROOT ----------

    signal oldHash[depth+1];
    oldHash[0] <== oldLeaf;

    component oldHashers[depth];

    signal leftOld[depth];
    signal rightOld[depth];
    signal aOld[depth];
    signal bOld[depth];
    signal cOld[depth];
    signal dOld[depth];

    for (var i = 0; i < depth; i++) {

        pathIndices[i] * (pathIndices[i] - 1) === 0;

        oldHashers[i] = Poseidon(2);

        aOld[i] <== oldHash[i] * (1 - pathIndices[i]);
        bOld[i] <== pathElements[i] * pathIndices[i];
        leftOld[i] <== aOld[i] + bOld[i];

        cOld[i] <== pathElements[i] * (1 - pathIndices[i]);
        dOld[i] <== oldHash[i] * pathIndices[i];
        rightOld[i] <== cOld[i] + dOld[i];

        oldHashers[i].inputs[0] <== leftOld[i];
        oldHashers[i].inputs[1] <== rightOld[i];

        oldHash[i+1] <== oldHashers[i].out;
    }

    oldHash[depth] === oldRoot;

    // ---------- NEW ROOT ----------

    signal newHash[depth+1];
    newHash[0] <== newLeaf;

    component newHashers[depth];

    signal leftNew[depth];
    signal rightNew[depth];
    signal aNew[depth];
    signal bNew[depth];
    signal cNew[depth];
    signal dNew[depth];

    for (var i = 0; i < depth; i++) {

        newHashers[i] = Poseidon(2);

        aNew[i] <== newHash[i] * (1 - pathIndices[i]);
        bNew[i] <== pathElements[i] * pathIndices[i];
        leftNew[i] <== aNew[i] + bNew[i];

        cNew[i] <== pathElements[i] * (1 - pathIndices[i]);
        dNew[i] <== newHash[i] * pathIndices[i];
        rightNew[i] <== cNew[i] + dNew[i];

        newHashers[i].inputs[0] <== leftNew[i];
        newHashers[i].inputs[1] <== rightNew[i];

        newHash[i+1] <== newHashers[i].out;
    }

    newHash[depth] === newRoot;

    isValid <== 1;
}

component main = MerkleUpdate(2);