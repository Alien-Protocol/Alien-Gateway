pragma circom 2.0.0;
include "../username_hash.circom";

// Constructs a leaf from a username
template UsernameLeaf() {
    signal input username;
    signal output leaf;

    component hasher = UsernameHasher();
    hasher.username <== username;
    
    leaf <== hasher.hash;
}
