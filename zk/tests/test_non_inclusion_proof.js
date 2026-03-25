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

const path = require("path");
const assert = require("assert");
const snarkjs = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

const CIRCUIT = "merkle_non_inclusion";
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
    const [left, right] = indices[i] === 0 ? [cur, siblings[i]] : [siblings[i], cur];
    cur = F.toObject(poseidon([left, right]));
  }
  return cur;
}

function usernameHash(poseidon, username) {
  const F = poseidon.F;

  // username_hash_impl:
  // - Poseidon(4) over chunks of 4 (8 chunks)
  // - Poseidon(4) over the 8 intermediate hashes grouped into 2 chunks of 4
  // - Poseidon(2) final
  const h = [];
  for (let i = 0; i < 8; i++) {
    h[i] = F.toObject(
      poseidon([
        username[i * 4 + 0],
        username[i * 4 + 1],
        username[i * 4 + 2],
        username[i * 4 + 3],
      ])
    );
  }

  const h2 = [];
  for (let i = 0; i < 2; i++) {
    const j = i * 4;
    h2[i] = F.toObject(poseidon([h[j + 0], h[j + 1], h[j + 2], h[j + 3]]));
  }

  return F.toObject(poseidon([h2[0], h2[1]]));
}

async function runTests() {
  const poseidon = await buildPoseidon();
  const emptyHashes = await buildEmptyHashes(poseidon, LEVELS);

  // Find a username_hash that fits into < 2^252 so the circuit's range check passes.
  const limit = 1n << 252n;
  let username = null;
  let h = null;
  for (let candidate = 1n; candidate < 5000n; candidate++) {
    username = new Array(32).fill(0n);
    username[0] = candidate;
    h = usernameHash(poseidon, username);
    if (h > 1n && h < limit - 2n) break;
  }
  assert.ok(username && h !== null, "failed to find username with hash < 2^252");

  // Build a tree where leaf_before = h-1 at index 0 and leaf_after = h+1 at index 1.
  const leaf_before = h - 1n;
  const leaf_after = h + 1n;

  // Indices are derived from pathIndices bits (LSB-first).
  const merklePathBeforeIndices = new Array(LEVELS).fill(0);
  const merklePathAfterIndices = new Array(LEVELS).fill(0);
  merklePathAfterIndices[0] = 1;

  // Siblings are empty subtree roots except at the first level where the two leaves are siblings.
  const merklePathBeforeSiblings = emptyHashes.slice(0, LEVELS).map((x) => x);
  const merklePathAfterSiblings = emptyHashes.slice(0, LEVELS).map((x) => x);
  merklePathBeforeSiblings[0] = leaf_after;
  merklePathAfterSiblings[0] = leaf_before;

  const root = computeRoot(poseidon, leaf_before, merklePathBeforeSiblings, merklePathBeforeIndices);

  const inputValid = {
    username: username.map((x) => x.toString()),
    leaf_before: leaf_before.toString(),
    leaf_after: leaf_after.toString(),
    merklePathBeforeSiblings: merklePathBeforeSiblings.map((x) => x.toString()),
    merklePathBeforeIndices: merklePathBeforeIndices.map((x) => x.toString()),
    merklePathAfterSiblings: merklePathAfterSiblings.map((x) => x.toString()),
    merklePathAfterIndices: merklePathAfterIndices.map((x) => x.toString()),
    root: root.toString(),
  };

  // ── Test 1: valid non-inclusion proof ─────────────────────────────────────
  process.stdout.write("\n── Test 1: valid non-inclusion proof ───────────────────────\n");
  {
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      inputValid,
      WASM_PATH,
      ZKEY_PATH
    );

    const vKey = require(VKEY_PATH);
    const valid = await snarkjs.groth16.verify(vKey, publicSignals, proof);
    assert.ok(valid, "proof should verify");

    assert.strictEqual(publicSignals[0], root.toString(), "publicSignals[0] must be root");
    assert.strictEqual(publicSignals[1], root.toString(), "publicSignals[1] must echo out_root=root");
    assert.strictEqual(
      publicSignals[publicSignals.length - 1],
      "1",
      "last public signal must be isAvailable=1"
    );
    process.stdout.write("  ✔  proof generated and verified\n");
  }

  // ── Test 2: taken username (username_hash == leaf_before) rejected ───
  process.stdout.write("\n── Test 2: occupied boundary rejected ───────────────────────\n");
  {
    const leaf_before_taken = h; // equals username_hash -> strict inequality must fail
    const leaf_after_taken = h + 1n;

    const merklePathBeforeSiblingsTaken = emptyHashes.slice(0, LEVELS).map((x) => x);
    const merklePathAfterSiblingsTaken = emptyHashes.slice(0, LEVELS).map((x) => x);
    merklePathBeforeSiblingsTaken[0] = leaf_after_taken;
    merklePathAfterSiblingsTaken[0] = leaf_before_taken;

    const rootTaken = computeRoot(
      poseidon,
      leaf_before_taken,
      merklePathBeforeSiblingsTaken,
      merklePathBeforeIndices
    );

    const inputTaken = {
      username: username.map((x) => x.toString()),
      leaf_before: leaf_before_taken.toString(),
      leaf_after: leaf_after_taken.toString(),
      merklePathBeforeSiblings: merklePathBeforeSiblingsTaken.map((x) => x.toString()),
      merklePathBeforeIndices: merklePathBeforeIndices.map((x) => x.toString()),
      merklePathAfterSiblings: merklePathAfterSiblingsTaken.map((x) => x.toString()),
      merklePathAfterIndices: merklePathAfterIndices.map((x) => x.toString()),
      root: rootTaken.toString(),
    };

    let threw = false;
    try {
      await snarkjs.groth16.fullProve(inputTaken, WASM_PATH, ZKEY_PATH);
    } catch {
      threw = true;
    }
    assert.ok(threw, "witness generation/proof should fail when username_hash equals a boundary leaf");
    process.stdout.write("  ✔  boundary-equality correctly rejected\n");
  }

  // ── Test 3: tampered root rejected ───────────────────────────────────────
  process.stdout.write("\n── Test 3: tampered root rejected ──────────────────────────\n");
  {
    const inputBadRoot = { ...inputValid, root: (root + 1n).toString() };
    let threw = false;
    try {
      await snarkjs.groth16.fullProve(inputBadRoot, WASM_PATH, ZKEY_PATH);
    } catch {
      threw = true;
    }
    assert.ok(threw, "witness generation should fail for a tampered root");
    process.stdout.write("  ✔  tampered root correctly rejected\n");
  }

  process.stdout.write("\n══ All tests passed ══\n\n");
}

runTests().catch((err) => {
  process.stderr.write(`\n✘  Test failed: ${err.message ?? err}\n`);
  process.exit(1);
});

