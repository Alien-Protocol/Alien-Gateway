pragma circom 2.0.0;
include "username_leaf.circom";
include "merkle_path.circom";

// Merkle Inclusion Proof Circuit
template MerkleInclusionProof(levels) {
    signal input username[32]; // private username
    signal input pathElements[levels];
    signal input pathIndices[levels];
    signal input root; // expected root

    signal output out_root;
    signal output isValid;

    component leafGen = UsernameLeaf();
    for (var i = 0; i < 32; i++) { leafGen.username[i] <== username[i]; }

    component pathVerifier = MerklePathVerifier(levels);
    pathVerifier.leaf <== leafGen.leaf;
    pathVerifier.root <== root;
    for (var i = 0; i < levels; i++) {
        pathVerifier.pathElements[i] <== pathElements[i];
        pathVerifier.pathIndices[i] <== pathIndices[i];
    }

    // Since the verifier enforces the equality using ===
    // If we get here, the path is valid.
    out_root <== root;
    // AUDIT NOTE (F-07): isValid is hardcoded to 1 and carries no information.
    // The actual soundness guarantee comes from the root === calculator.root
    // constraint in MerklePathVerifier. Consider removing this output signal
    // to avoid misleading callers into treating it as a computed result.
    isValid <== 1;
}

// Configurable component instance (e.g. 20 levels for 1000+ users)
component main {public [root]} = MerkleInclusionProof(20);
