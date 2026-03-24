pragma circom 2.0.0;

include "path_calculator.circom";

// MerkleNonInclusionProof
//
// Proves that a username slot is EMPTY in the Sparse Merkle Tree, i.e.
// the leaf at the given path is 0, without revealing the username itself.
//
// The prover supplies the Merkle path to the target slot and the current
// root. The circuit recomputes the root from leaf = 0 and asserts it
// matches the public root, proving the slot is unoccupied.
//
// Public inputs  : root
// Private inputs : merklePathSiblings, merklePathIndices
// Public output  : out_root  (echoes root, for on-chain anchoring)

template MerkleNonInclusionProof(levels) {

    // ── Private inputs ───────────────────────────────────────────────────────
    signal input merklePathSiblings[levels];  // sibling at each tree level
    signal input merklePathIndices[levels];   // 0 = left child, 1 = right child

    // ── Public inputs ────────────────────────────────────────────────────────
    signal input root;  // current SMT root

    // ── Public output ────────────────────────────────────────────────────────
    signal output out_root;

    // ── Verify the slot is empty ─────────────────────────────────────────────
    // Walk up from leaf = 0 (empty slot) along the provided path.
    // The computed root must equal the public root, proving the slot is empty.
    component calc = PathCalculator(levels);
    calc.leaf <== 0;
    for (var i = 0; i < levels; i++) {
        calc.pathElements[i] <== merklePathSiblings[i];
        calc.pathIndices[i]  <== merklePathIndices[i];
    }

    calc.root === root;

    out_root <== root;
}

component main {public [root]} = MerkleNonInclusionProof(20);
