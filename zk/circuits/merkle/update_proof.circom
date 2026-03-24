pragma circom 2.0.0;

include "path_calculator.circom";

// MerkleUpdateProof
//
// Proves a valid state transition when a new username commitment is inserted
// into the Sparse Merkle Tree:
//
//   oldRoot  (slot was empty, leaf == 0)
//     →  newRoot  (slot now holds usernameHash)
//
// The same Merkle path is used for both computations, proving that only the
// target leaf changed and all siblings remain unchanged.
//
// Public inputs  : oldRoot, newRoot
// Private inputs : usernameHash, merklePathSiblings, merklePathIndices
// Public output  : out_newRoot  (equals newRoot, for on-chain anchoring)

template MerkleUpdateProof(levels) {

    // ── Private inputs ───────────────────────────────────────────────────────
    signal input usernameHash;                    // hash of the new username leaf
    signal input merklePathSiblings[levels];      // sibling node at each tree level
    signal input merklePathIndices[levels];       // 0 = current node is left child
                                                  // 1 = current node is right child

    // ── Public inputs ────────────────────────────────────────────────────────
    signal input oldRoot;   // Merkle root before insertion (slot was 0)
    signal input newRoot;   // Merkle root after  insertion

    // ── Public output ────────────────────────────────────────────────────────
    signal output out_newRoot;

    // ── Verify old root ──────────────────────────────────────────────────────
    // Compute the root reached by walking up from an empty leaf (0) along the
    // provided path.  This must equal oldRoot, proving the slot was unoccupied.
    component oldCalc = PathCalculator(levels);
    oldCalc.leaf <== 0;
    for (var i = 0; i < levels; i++) {
        oldCalc.pathElements[i] <== merklePathSiblings[i];
        oldCalc.pathIndices[i]  <== merklePathIndices[i];
    }
    oldCalc.root === oldRoot;

    // ── Verify new root ──────────────────────────────────────────────────────
    // Compute the root reached by walking up from usernameHash along the same
    // path.  This must equal newRoot, proving the transition is correct.
    component newCalc = PathCalculator(levels);
    newCalc.leaf <== usernameHash;
    for (var i = 0; i < levels; i++) {
        newCalc.pathElements[i] <== merklePathSiblings[i];
        newCalc.pathIndices[i]  <== merklePathIndices[i];
    }
    newCalc.root === newRoot;

    // ── Output new root for on-chain anchoring ───────────────────────────────
    out_newRoot <== newRoot;
}

component main {public [oldRoot, newRoot]} = MerkleUpdateProof(20);
