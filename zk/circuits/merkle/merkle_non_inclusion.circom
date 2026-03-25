pragma circom 2.0.0;

include "sorted_tree.circom";
include "range_check.circom";
include "../username_hash_impl.circom";

// MerkleNonInclusionProof
//
// Proves that a (private) username_hash is NOT contained in a sorted Merkle
// tree by providing two consecutive boundary leaves:
//
//   leaf_before < username_hash < leaf_after
//
// Soundness comes from:
// 1) verifying both boundary leaves exist in the tree at consecutive indices
// 2) checking strict range ordering around username_hash
//
// Public inputs  : root
// Public outputs : out_root (echoes root), isAvailable (1 if non-inclusion holds)
// Private inputs : username, boundary leaves + their Merkle paths
template MerkleNonInclusionProof(levels) {
    // Private inputs
    signal input username[32];

    signal input leaf_before;
    signal input leaf_after;

    signal input merklePathBeforeSiblings[levels];
    signal input merklePathBeforeIndices[levels];
    signal input merklePathAfterSiblings[levels];
    signal input merklePathAfterIndices[levels];

    // Public inputs
    signal input root;

    // Public outputs
    signal output out_root;
    signal output isAvailable;

    // 1) Compute username_hash from the private username.
    component usernameHasher = UsernameHash();
    for (var i = 0; i < 32; i++) {
        usernameHasher.username[i] <== username[i];
    }

    // 2) Verify boundaries exist and are consecutive.
    component tree = SortedTreeConsecutive(levels);
    tree.root <== root;
    tree.leaf_before <== leaf_before;
    tree.leaf_after <== leaf_after;
    for (var i = 0; i < levels; i++) {
        tree.merklePathBeforeSiblings[i] <== merklePathBeforeSiblings[i];
        tree.merklePathBeforeIndices[i] <== merklePathBeforeIndices[i];
        tree.merklePathAfterSiblings[i] <== merklePathAfterSiblings[i];
        tree.merklePathAfterIndices[i] <== merklePathAfterIndices[i];
    }

    // 3) Prove username_hash falls strictly between the consecutive boundaries.
    component range = RangeCheck(252);
    range.leaf_before <== leaf_before;
    range.value <== usernameHasher.username_hash;
    range.leaf_after <== leaf_after;
    range.inRange === 1;

    // 4) Output availability.
    out_root <== root;
    isAvailable <== 1;
}

component main {public [root]} = MerkleNonInclusionProof(20);

