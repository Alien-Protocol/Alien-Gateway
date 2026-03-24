"use strict";

/**
 * test_non_inclusion_proof.js
 *
 * Witness generation + Groth16 proof test for MerkleNonInclusionProof.
 *
 * Prerequisites (run from zk/):
 *   npm run compile:merkle_non_inclusion
 *   npm run setup:merkle_non_inclusion
 *
 * Run:
 *   node tests/test_non_inclusion_proof.js
 */

const path    = require("path");
const assert  = require("assert");
const snarkjs = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

const CIRCUIT   = "merkle_non_inclusion";
const BUILD_DIR = path.join(__dirname, "..", "build", CIRCUIT);
const WASM_PATH = path.join(BUILD_DIR, "wasm", `${CIRCUIT}_js`, `${CIRCUIT}.wasm`);
const ZKEY_PATH = path.join(BUILD_DIR, `${CIRCUIT}_final.zkey`);
const VKEY_PATH = path.join(BUILD_DIR, "verification_key.json");

const LEVELS = 20;

// Build empty-subtree hash table: emptyHashes[i] = root of all-zero tree of height i
async function buildEmptyHashes(poseidon, depth) {
    const F = poseidon.F;
    const h = [BigInt(0)];
    for (let i = 0; i < depth; i++) {
        h.push(F.toObject(poseidon([h[i], h[i]])));
    }
    return h;
}

// Recompute root from leaf + path (mirrors PathCalculator / BitSelector logic)
function computeRoot(poseidon, leaf, siblings, indices) {
    const F = poseidon.F;
    let cur = leaf;
    for (let i = 0; i < siblings.length; i++) {
        const [left, right] = indices[i] === 0
            ? [cur, siblings[i]]
            : [siblings[i], cur];
        cur = F.toObject(poseidon([left, right]));
    }
    return cur;
}

async function runTests() {
    const poseidon    = await buildPoseidon();
    const emptyHashes = await buildEmptyHashes(poseidon, LEVELS);

    // All-empty tree: position 0, every index = 0, sibling = empty subtree of that height
    const siblings = emptyHashes.slice(0, LEVELS);
    const indices  = new Array(LEVELS).fill(0);
    const root     = computeRoot(poseidon, BigInt(0), siblings, indices);

    assert.strictEqual(
        root.toString(),
        emptyHashes[LEVELS].toString(),
        "computed root should match all-empty tree root"
    );

    const input = {
        merklePathSiblings: siblings.map(x => x.toString()),
        merklePathIndices:  indices.map(x => x.toString()),
        root:               root.toString(),
    };

    // ── Test 1: valid non-inclusion proof ─────────────────────────────────────
    process.stdout.write("\n── Test 1: valid non-inclusion proof ───────────────────────\n");
    {
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            input, WASM_PATH, ZKEY_PATH
        );
        const vKey  = require(VKEY_PATH);
        const valid = await snarkjs.groth16.verify(vKey, publicSignals, proof);

        assert.ok(valid, "proof should verify");
        assert.strictEqual(publicSignals[0], root.toString(), "out_root must equal root");
        process.stdout.write("  ✔  proof generated and verified\n");
        process.stdout.write(`  ✔  out_root = ${publicSignals[0]}\n`);
    }

    // ── Test 2: occupied slot → witness generation must fail ──────────────────
    // Simulate a slot that holds a real leaf (non-zero) by computing the root
    // from a non-zero leaf and passing that as the public root while keeping
    // the path pointing to leaf = 0. The constraint calc.root === root fails.
    process.stdout.write("\n── Test 2: occupied slot rejected ──────────────────────────\n");
    {
        const fakeLeaf = poseidon.F.toObject(poseidon([BigInt(99)]));
        const occupiedRoot = computeRoot(poseidon, fakeLeaf, siblings, indices);
        const badInput = { ...input, root: occupiedRoot.toString() };
        let threw = false;
        try {
            await snarkjs.groth16.fullProve(badInput, WASM_PATH, ZKEY_PATH);
        } catch {
            threw = true;
        }
        assert.ok(threw, "witness generation should fail when slot is occupied");
        process.stdout.write("  ✔  occupied slot correctly rejected\n");
    }

    // ── Test 3: tampered root → witness generation must fail ──────────────────
    process.stdout.write("\n── Test 3: tampered root rejected ──────────────────────────\n");
    {
        const badInput = { ...input, root: (root + BigInt(1)).toString() };
        let threw = false;
        try {
            await snarkjs.groth16.fullProve(badInput, WASM_PATH, ZKEY_PATH);
        } catch {
            threw = true;
        }
        assert.ok(threw, "witness generation should fail for a tampered root");
        process.stdout.write("  ✔  tampered root correctly rejected\n");
    }

    process.stdout.write("\n══ All tests passed ══\n\n");
}

runTests().catch(err => {
    process.stderr.write(`\n✘  Test failed: ${err.message ?? err}\n`);
    process.exit(1);
});
