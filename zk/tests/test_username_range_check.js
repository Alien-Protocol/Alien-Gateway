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
  console.log("=== username_hash range-check tests ===\n");

  // ── Test 1: valid ASCII username should generate a witness ────────────────
  {
    const input = { username: encodeUsername("alice") };
    try {
      const { wtns } = await snarkjs.wtns.calculate(input, WASM_PATH, {});
      assert.ok(wtns, "Witness should be generated for valid ASCII input");
      console.log("PASS  Test 1: valid ASCII username ('alice') accepted");
    } catch (err) {
      console.error("FAIL  Test 1:", err.message);
      process.exitCode = 1;
    }
  }

  // ── Test 2: all-zero (null bytes) input should be accepted (0 <= 127) ─────
  {
    const input = { username: new Array(32).fill(0) };
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
      console.log("PASS  Test 2: all-zero username accepted (0 is valid ASCII)");
    } catch (err) {
      console.error("FAIL  Test 2:", err.message);
      process.exitCode = 1;
    }
  }

  // ── Test 3: value 127 (DEL) is the boundary — must be accepted ───────────
  {
    const input = { username: new Array(32).fill(127) };
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
      console.log("PASS  Test 3: boundary value 127 accepted");
    } catch (err) {
      console.error("FAIL  Test 3:", err.message);
      process.exitCode = 1;
    }
  }

  // ── Test 4: value 128 must be rejected by the circuit ────────────────────
  {
    const input = { username: new Array(32).fill(0) };
    input.username[0] = 128; // out of range
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
      console.error(
        "FAIL  Test 4: value 128 should have been rejected but was accepted"
      );
      process.exitCode = 1;
    } catch (err) {
      console.log(
        "PASS  Test 4: value 128 correctly rejected by range constraint"
      );
    }
  }

  // ── Test 5: large out-of-range value (255) must be rejected ───────────────
  {
    const input = { username: new Array(32).fill(0) };
    input.username[15] = 255;
    try {
      await snarkjs.wtns.calculate(input, WASM_PATH, {});
      console.error(
        "FAIL  Test 5: value 255 should have been rejected but was accepted"
      );
      process.exitCode = 1;
    } catch (err) {
      console.log(
        "PASS  Test 5: value 255 correctly rejected by range constraint"
      );
    }
  }

  console.log("\n=== done ===");
}

main().catch((err) => {
  console.error("Unexpected error:", err);
  process.exitCode = 1;
});
