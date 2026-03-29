"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashUsername = hashUsername;
exports.bigintToBytes32 = bigintToBytes32;
exports.encodeUsername = encodeUsername;
// @ts-ignore - circomlibjs doesn't have TypeScript definitions
const { buildPoseidon } = require("circomlibjs");
const USERNAME_SIGNAL_LENGTH = 32;
async function hashUsername(username) {
    const poseidon = await buildPoseidon();
    const field = poseidon.F;
    const encoded = encodeUsername(username);
    const levelOne = [];
    for (let index = 0; index < 8; index += 1) {
        levelOne.push(poseidon(encoded.slice(index * 4, index * 4 + 4)));
    }
    const levelTwo = [];
    for (let index = 0; index < 2; index += 1) {
        levelTwo.push(poseidon(levelOne.slice(index * 4, index * 4 + 4)));
    }
    const commitment = field.toObject(poseidon([levelTwo[0], levelTwo[1]]));
    return bigintToBytes32(commitment);
}
function bigintToBytes32(value) {
    return `0x${value.toString(16).padStart(64, "0")}`;
}
function encodeUsername(username) {
    if (username.length === 0) {
        throw new Error("Username must not be empty.");
    }
    const encoded = Array.from(username).map((character) => BigInt(character.charCodeAt(0)));
    if (encoded.length > USERNAME_SIGNAL_LENGTH) {
        throw new Error(`Username must be ${USERNAME_SIGNAL_LENGTH} characters or fewer.`);
    }
    while (encoded.length < USERNAME_SIGNAL_LENGTH) {
        encoded.push(0n);
    }
    return encoded;
}
