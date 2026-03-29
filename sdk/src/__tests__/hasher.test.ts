import assert from "node:assert/strict";
import test from "node:test";

import { buildPoseidon } from "circomlibjs";

import { UsernameHasher } from "../hasher";

// Reference implementation from test_non_inclusion_proof.js
function usernameHash(poseidon: any, username: bigint[]): bigint {
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

test("UsernameHasher.hash matches circuit output for 'alice'", async () => {
  const hasher = await UsernameHasher.create();

  // Encode 'alice' as per the specification: ASCII values, zero-padded to 32 elements
  const aliceEncoded = [
    97, 108, 105, 99, 101, // 'a', 'l', 'i', 'c', 'e'
    ...new Array(27).fill(0), // zero padding
  ];

  const hash = hasher.hash("alice");
  const expected = usernameHash(await buildPoseidon(), aliceEncoded.map(BigInt));

  assert.strictEqual(hash, expected, "Hash should match reference implementation");
});

test("UsernameHasher.hashRaw matches reference implementation", async () => {
  const hasher = await UsernameHasher.create();
  const poseidon = await buildPoseidon();

  // Test with a sample encoded username
  const username = new Array(32).fill(0n);
  username[0] = 1n;
  username[1] = 2n;
  username[2] = 3n;

  const hash = hasher.hashRaw(username.map(Number));
  const expected = usernameHash(poseidon, username);

  assert.strictEqual(hash, expected, "hashRaw should match reference implementation");
});

test("UsernameHasher.hash encodes username correctly", async () => {
  const hasher = await UsernameHasher.create();

  // Test encoding
  const hash = hasher.hash("abc");
  const expectedEncoded = [97, 98, 99, ...new Array(29).fill(0)]; // 'a', 'b', 'c', zeros
  const expectedHash = hasher.hashRaw(expectedEncoded);

  assert.strictEqual(hash, expectedHash, "hash should encode username correctly");
});

test("UsernameHasher.hash throws for username > 32 characters", async () => {
  const hasher = await UsernameHasher.create();

  const longUsername = "a".repeat(33);

  assert.throws(
    () => hasher.hash(longUsername),
    { message: "Username must be 32 characters or less" }
  );
});

test("UsernameHasher.hashRaw throws for array != 32 elements", async () => {
  const hasher = await UsernameHasher.create();

  assert.throws(
    () => hasher.hashRaw(new Array(31).fill(0)),
    { message: "Username array must contain exactly 32 elements" }
  );

  assert.throws(
    () => hasher.hashRaw(new Array(33).fill(0)),
    { message: "Username array must contain exactly 32 elements" }
  );
});

test("UsernameHasher.hash throws for non-ASCII characters", async () => {
  const hasher = await UsernameHasher.create();

  assert.throws(
    () => hasher.hash("test©"),
    { message: "Username contains non-ASCII characters" }
  );
});