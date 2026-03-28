"use strict";

/**
 * test_update_proof.js
 *
 * Witness generation + Groth16 proof test for the MerkleUpdateProof circuit.
 *
 * Prerequisites (run from zk/):
 *   npm run compile          # compiles all circuits incl. merkle_update_proof
 *   npm run setup            # runs trusted setup for all circuits
 *
 * Run:
 *   node tests/test_update_proof.js
 */

const path    = require("path");
const assert  = require("assert");
const snarkjs = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

// ── Paths ────────────────────────────────────────────────────────────────────

const CIRCUIT     = "merkle_update_proof";
const BUILD_DIR   = path.join(__dirname, "..", "build", CIRCUIT);
const WASM_PATH   = path.join(BUILD_DIR, "wasm", `${CIRCUIT}_js`, `${CIRCUIT}.wasm`);
const ZKEY_PATH   = path.join(BUILD_DIR, `${CIRCUIT}_final.zkey`);
const VKEY_PATH   = path.join(BUILD_DIR, "verification_key.json");

const LEVELS = 20;

// ── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Build the empty-subtree hash table for a Poseidon SMT of `depth` levels.
 * emptyHashes[0] = 0  (empty leaf)
 * emptyHashes[i] = Poseidon(emptyHashes[i-1], emptyHashes[i-1])
 */
async function buildEmptyHashes(poseidon, depth) {
    const F = poseidon.F;
    const h = [BigInt(0)];
    for (let i = 0; i < depth; i++) {
        const node = poseidon([h[i], h[i]]);
        h.push(F.toObject(node));
    }
    return h;
}

/**
 * Compute the Merkle root from a leaf using the BitSelector convention:
 *   index == 0  →  Poseidon(current, sibling)
 *   index == 1  →  Poseidon(sibling,  current)
 */
function computeRoot(poseidon, leaf, siblings, indices) {
    const F = poseidon.F;
    let current = leaf;
    for (let i = 0; i < siblings.length; i++) {
        let left, right;
        if (indices[i] === 0) {
            left  = current;
            right = siblings[i];
        } else {
            left  = siblings[i];
            right = current;
        }
        current = F.toObject(poseidon([left, right]));
    }
    return current;
}

/**
 * Compute username hash using the same algorithm as the UsernameHash circuit
 * This mirrors the 2-level Poseidon hashing approach used in username_hash.circom
 */
function computeUsernameHash(poseidon, username) {
    const F = poseidon.F;
    
    // Step 1: Hash in chunks of 4 (8 chunks total)
    const h = [];
    for (let i = 0; i < 8; i++) {
        const chunk = [];
        for (let j = 0; j < 4; j++) {
            chunk.push(username[i * 4 + j]);
        }
        h.push(F.toObject(poseidon(chunk)));
    }
    
    // Step 2: Hash intermediate hashes (2 chunks of 4)
    const h2 = [];
    for (let i = 0; i < 2; i++) {
        const chunk = [];
        for (let j = 0; j < 4; j++) {
            chunk.push(h[i * 4 + j]);
        }
        h2.push(F.toObject(poseidon(chunk)));
    }
    
    // Final hash
    const finalHash = F.toObject(poseidon(h2));
    return finalHash;
}

// ── Test runner ──────────────────────────────────────────────────────────────

async function runTests() {
    const poseidon = await buildPoseidon();
    const F        = poseidon.F;

    const emptyHashes = await buildEmptyHashes(poseidon, LEVELS);

    // ── Fixtures ─────────────────────────────────────────────────────────────
    // Insert at position 0 (all indices == 0, all siblings are empty subtrees)
    const merklePathSiblings = emptyHashes.slice(1, LEVELS + 1); // siblings[i] = emptyHashes[i+1]?
    // Actually: at level i from the leaf, the sibling of the LEFT child is the
    // right subtree of the same depth, which is emptyHashes[i].
    // Let's rebuild correctly:
    //   position 0 means at every level, current node is the LEFT child (index=0),
    //   so sibling is the RIGHT child which is an all-empty subtree of height i.
    const siblings = emptyHashes.slice(0, LEVELS); // siblings[i] = emptyHashes[i]
    const indices  = new Array(LEVELS).fill(0);

    const oldRoot = computeRoot(poseidon, BigInt(0), siblings, indices);
    // oldRoot must equal emptyHashes[LEVELS] (root of all-zero tree)
    assert.strictEqual(
        oldRoot.toString(),
        emptyHashes[LEVELS].toString(),
        "oldRoot should match pre-computed all-empty root"
    );

    // Use a simple username array (in a real flow this comes from user input)
    // Create a 32-character username array with simple values
    const username = new Array(32).fill(BigInt(42)); // Simple test username
    // Compute the expected username hash using the same algorithm as the circuit
    const usernameHash = computeUsernameHash(poseidon, username);
    const newRoot      = computeRoot(poseidon, usernameHash, siblings, indices);

    const input = {
        username:            username.map(x => x.toString()),
        merklePathSiblings:  siblings.map(x => x.toString()),
        merklePathIndices:   indices.map(x => x.toString()),
        oldRoot:             oldRoot.toString(),
        newRoot:             newRoot.toString(),
    };

    // ── Test 1: valid insert ──────────────────────────────────────────────────
    process.stdout.write("\n── Test 1: valid insert proof ──────────────────────────────\n");
    {
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            input,
            WASM_PATH,
            ZKEY_PATH
        );

        const vKey  = require(VKEY_PATH);
        const valid = await snarkjs.groth16.verify(vKey, publicSignals, proof);

        assert.ok(valid, "Proof should be valid for a correct insert");
        process.stdout.write("  ✔  proof generated and verified\n");
        process.stdout.write(`  ✔  out_newRoot = ${publicSignals[0]}\n`);
        assert.strictEqual(
            publicSignals[0],
            newRoot.toString(),
            "out_newRoot public signal must equal newRoot"
        );
        process.stdout.write("  ✔  out_newRoot matches expected newRoot\n");
    }

    // ── Test 2: tampered newRoot → witness generation must fail ──────────────
    process.stdout.write("\n── Test 2: tampered newRoot rejected ───────────────────────\n");
    {
        const badInput = { ...input, newRoot: (newRoot + BigInt(1)).toString() };
        let threw = false;
        try {
            await snarkjs.groth16.fullProve(badInput, WASM_PATH, ZKEY_PATH);
        } catch {
            threw = true;
        }
        assert.ok(threw, "Witness generation should fail for a tampered newRoot");
        process.stdout.write("  ✔  tampered newRoot correctly rejected\n");
    }

    // ── Test 3: tampered oldRoot → witness generation must fail ──────────────
    process.stdout.write("\n── Test 3: tampered oldRoot rejected ───────────────────────\n");
    {
        const badInput = { ...input, oldRoot: (oldRoot + BigInt(1)).toString() };
        let threw = false;
        try {
            await snarkjs.groth16.fullProve(badInput, WASM_PATH, ZKEY_PATH);
        } catch {
            threw = true;
        }
        assert.ok(threw, "Witness generation should fail for a tampered oldRoot");
        process.stdout.write("  ✔  tampered oldRoot correctly rejected\n");
    }

    process.stdout.write("\n══ All tests passed ══\n\n");
}

runTests().catch(err => {
    process.stderr.write(`\n✘  Test failed: ${err.message ?? err}\n`);
    process.exit(1);
});
