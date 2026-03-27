"use strict";

/**
 * test_username_range_check.js
 *
 * Verifies that username_hash circuit enforces the [0, 127] ASCII range
 * constraint on every username character (Fix for Finding F-03).
 *
 * Prerequisites (run from zk/):
 *   npm run compile   # compiles username_hash circuit
 *
 * Run:
 *   node tests/test_username_range_check.js
 */

const path = require("path");
const assert = require("assert");
const snarkjs = require("snarkjs");

const CIRCUIT = "username_hash";
const BUILD_DIR = path.join(__dirname, "..", "build", CIRCUIT);
const WASM_PATH = path.join(
  BUILD_DIR,
  "wasm",
  `${CIRCUIT}_js`,
  `${CIRCUIT}.wasm`
);

/** Encodes a string into a zero-padded 32-element ASCII array. */
function encodeUsername(str) {
  const arr = new Array(32).fill(0);
  for (let i = 0; i < Math.min(str.length, 32); i++) {
    arr[i] = str.charCodeAt(i);
  }
  return arr;
}

async function main() {
  // Test 1: valid ASCII username should generate a witness
  {
    const input = { username: encodeUsername("alice") };
    const { wtns } = await snarkjs.wtns.calculate(input, WASM_PATH, {});
    assert.ok(wtns, "Witness should be generated for valid ASCII input");
  }

  // Test 2: all-zero (null bytes) input should be accepted (0 <= 127)
  {
    const input = { username: new Array(32).fill(0) };
    await snarkjs.wtns.calculate(input, WASM_PATH, {});
  }

  // Test 3: value 127 (DEL) is the boundary — must be accepted
  {
    const input = { username: new Array(32).fill(127) };
    await snarkjs.wtns.calculate(input, WASM_PATH, {});
  }

  // Test 4: value 128 must be rejected by the circuit
  {
    const input = { username: new Array(32).fill(0) };
    input.username[0] = 128;
    let rejected = false;
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
    } catch (_) {
      rejected = true;
    }
    assert.ok(rejected, "Value 128 should be rejected by range constraint");
  }

  // Test 5: large out-of-range value (255) must be rejected
  {
    const input = { username: new Array(32).fill(0) };
    input.username[15] = 255;
    let rejected = false;
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
    } catch (_) {
      rejected = true;
    }
    assert.ok(rejected, "Value 255 should be rejected by range constraint");
  }
}

main().catch((err) => {
  process.stderr.write(`Unexpected error: ${err}\n`);
  process.exitCode = 1;
});
