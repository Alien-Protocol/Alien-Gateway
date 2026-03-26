const { buildPoseidon } = require('circomlibjs');

// Encodes a username string as a 32-byte zero-padded array of char codes.
function encodeUsername(username) {
  const arr = Array.from(username).map((c) => c.charCodeAt(0));
  while (arr.length < 32) arr.push(0);
  return arr.slice(0, 32);
}

// Hashes a username using the Poseidon hash function instance, matching the Circom circuit output.
async function hashUsername(username) {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;
  const input = encodeUsername(username);
  const h = [];
  for (let i = 0; i < 8; i++) {
    h[i] = poseidon(input.slice(i * 4, i * 4 + 4));
  }
  const h2 = [];
  for (let i = 0; i < 2; i++) {
    h2[i] = poseidon(h.slice(i * 4, i * 4 + 4));
  }
  // Convert Poseidon output to bigint
  return F.toObject(poseidon([h2[0], h2[1]]));
}

// Converts a bigint to a 32-byte hex string (BytesN<32>), zero-padded.
function bigintToBytes32(value) {
  let hex = value.toString(16);
  while (hex.length < 64) hex = '0' + hex;
  return '0x' + hex;
}

module.exports = { hashUsername, bigintToBytes32 };
