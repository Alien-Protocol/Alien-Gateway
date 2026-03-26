pragma circom 2.0.0;
include "../username_hash_impl.circom";

// Constructs a leaf from a username
template UsernameLeaf() {
    signal input username[32];
    signal output leaf;

    component hasher = UsernameHash();
    for (var i = 0; i < 32; i++) {
        hasher.username[i] <== username[i];
    }

    leaf <== hasher.username_hash;
}
