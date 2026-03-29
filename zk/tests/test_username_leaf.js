"use strict";

const path = require("path");
const assert = require("assert");
const fs = require("fs");
const snarkjs = require("snarkjs");
const { buildPoseidon } = require("circomlibjs");

const CIRCUIT = "username_leaf_main";
const BUILD_DIR = path.join(__dirname, "..", "build", CIRCUIT);
const WASM_PATH = path.join(
  BUILD_DIR,
  "wasm",
  `${CIRCUIT}_js`,
  `${CIRCUIT}.wasm`
);
const WITNESS_PATH = path.join(BUILD_DIR, "wtns", "witness.wtns");

function encodeUsername(username) {
  const bytes = new Array(32).fill(0);
  for (let i = 0; i < Math.min(username.length, 32); i++) {
    bytes[i] = username.charCodeAt(i);
  }
  return bytes;
}

async function computeUsernameHash(username) {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const bytes = encodeUsername(username);

  const h = [];
  for (let i = 0; i < 8; i++) {
    const chunk = [
      BigInt(bytes[i * 4]),
      BigInt(bytes[i * 4 + 1]),
      BigInt(bytes[i * 4 + 2]),
      BigInt(bytes[i * 4 + 3])
    ];
    h.push(F.toObject(poseidon(chunk)));
  }

  const h2 = [];
  for (let i = 0; i < 2; i++) {
    const chunk = [h[i * 4], h[i * 4 + 1], h[i * 4 + 2], h[i * 4 + 3]];
    h2.push(F.toObject(poseidon(chunk)));
  }

  return F.toObject(poseidon([h2[0], h2[1]]));
}

async function main() {
  const username = "amar";
  const input = { username: encodeUsername(username) };
  const expectedHash = await computeUsernameHash(username);

  fs.mkdirSync(path.dirname(WITNESS_PATH), { recursive: true });
  await snarkjs.wtns.calculate(input, WASM_PATH, WITNESS_PATH, {});
  const witness = await snarkjs.wtns.exportJson(WITNESS_PATH);
  const circuitHash = BigInt(witness[1]);

  assert.strictEqual(
    circuitHash.toString(),
    expectedHash.toString(),
    "Circuit hash should match Poseidon hash"
  );
}

main().catch((err) => {
  process.stderr.write(`Unexpected error: ${err}\n`);
  process.exitCode = 1;
});
