import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { buildPoseidon } from "circomlibjs";
import snarkjs from "snarkjs";

import { MerkleProofGenerator } from "../proof";
import type { InclusionInput, MerkleProofGeneratorConfig, NonInclusionInput } from "../types";

const LEVELS = 20;
const BUILD_DIR = path.resolve(__dirname, "../../../build");

const config: MerkleProofGeneratorConfig = {
  inclusion: {
    wasmPath: path.join(
      BUILD_DIR,
      "merkle_inclusion",
      "wasm",
      "merkle_inclusion_js",
      "merkle_inclusion.wasm",
    ),
    zkeyPath: path.join(BUILD_DIR, "merkle_inclusion", "merkle_inclusion_final.zkey"),
  },
  nonInclusion: {
    wasmPath: path.join(
      BUILD_DIR,
      "merkle_non_inclusion",
      "wasm",
      "merkle_non_inclusion_js",
      "merkle_non_inclusion.wasm",
    ),
    zkeyPath: path.join(BUILD_DIR, "merkle_non_inclusion", "merkle_non_inclusion_final.zkey"),
  },
};

const missingArtifacts = [
  config.inclusion.wasmPath,
  config.inclusion.zkeyPath,
  config.nonInclusion.wasmPath,
  config.nonInclusion.zkeyPath,
].filter((artifactPath) => !fs.existsSync(artifactPath));

test(
  "MerkleProofGenerator.proveInclusion generates a verifiable proof",
  { skip: missingArtifacts.length > 0 ? `Missing circuit artifacts: ${missingArtifacts.join(", ")}` : false },
  async () => {
    const poseidon = await buildPoseidon();
    const emptyHashes = await buildEmptyHashes(poseidon, LEVELS);
    const { username, usernameHash } = findValidUsername(poseidon);
    const pathElements = emptyHashes.slice(0, LEVELS);
    const pathIndices = new Array<number>(LEVELS).fill(0);
    const root = computeRoot(poseidon, usernameHash, pathElements, pathIndices);

    const generator = new MerkleProofGenerator(config);
    const result = await generator.proveInclusion({
      username: username.map(toSignalString),
      pathElements: pathElements.map(toSignalString),
      pathIndices,
      root: root.toString(),
    } satisfies InclusionInput);

    const verificationKey = await snarkjs.zKey.exportVerificationKey(config.inclusion.zkeyPath);
    const isValid = await snarkjs.groth16.verify(verificationKey, result.publicSignals, result.proof);

    assert.equal(isValid, true);
    assert.deepEqual(result.publicSignals, [root.toString(), root.toString()]);
  },
);

test(
  "MerkleProofGenerator.proveNonInclusion generates a verifiable proof",
  { skip: missingArtifacts.length > 0 ? `Missing circuit artifacts: ${missingArtifacts.join(", ")}` : false },
  async () => {
    const poseidon = await buildPoseidon();
    const emptyHashes = await buildEmptyHashes(poseidon, LEVELS);
    const { username, usernameHash } = findValidUsername(poseidon);
    const leafBefore = usernameHash - 1n;
    const leafAfter = usernameHash + 1n;

    const merklePathBeforeIndices = new Array<number>(LEVELS).fill(0);
    const merklePathAfterIndices = new Array<number>(LEVELS).fill(0);
    merklePathAfterIndices[0] = 1;

    const merklePathBeforeSiblings = emptyHashes.slice(0, LEVELS);
    const merklePathAfterSiblings = emptyHashes.slice(0, LEVELS);
    merklePathBeforeSiblings[0] = leafAfter;
    merklePathAfterSiblings[0] = leafBefore;

    const root = computeRoot(poseidon, leafBefore, merklePathBeforeSiblings, merklePathBeforeIndices);

    const generator = new MerkleProofGenerator(config);
    const result = await generator.proveNonInclusion({
      username: username.map(toSignalString),
      leaf_before: leafBefore.toString(),
      leaf_after: leafAfter.toString(),
      merklePathBeforeSiblings: merklePathBeforeSiblings.map(toSignalString),
      merklePathBeforeIndices,
      merklePathAfterSiblings: merklePathAfterSiblings.map(toSignalString),
      merklePathAfterIndices,
      root: root.toString(),
    } satisfies NonInclusionInput);

    const verificationKey = await snarkjs.zKey.exportVerificationKey(config.nonInclusion.zkeyPath);
    const isValid = await snarkjs.groth16.verify(verificationKey, result.publicSignals, result.proof);

    assert.equal(isValid, true);
    assert.deepEqual(result.publicSignals, [root.toString(), root.toString(), "1"]);
  },
);

type Poseidon = Awaited<ReturnType<typeof buildPoseidon>>;

async function buildEmptyHashes(poseidon: Poseidon, depth: number): Promise<bigint[]> {
  const hashes = [0n];

  for (let index = 0; index < depth; index += 1) {
    hashes.push(poseidon.F.toObject(poseidon([hashes[index], hashes[index]])));
  }

  return hashes;
}

function computeRoot(
  poseidon: Poseidon,
  leaf: bigint,
  siblings: bigint[],
  indices: number[],
): bigint {
  return siblings.reduce<bigint>((current, sibling, index) => {
    const inputs = indices[index] === 0 ? [current, sibling] : [sibling, current];
    return poseidon.F.toObject(poseidon(inputs));
  }, leaf);
}

function findValidUsername(poseidon: Poseidon): { username: bigint[]; usernameHash: bigint } {
  const limit = 1n << 252n;

  for (let candidate = 1n; candidate < 5000n; candidate += 1n) {
    const username = new Array<bigint>(32).fill(0n);
    username[0] = candidate;
    const usernameHash = hashUsername(poseidon, username);

    if (usernameHash > 1n && usernameHash < limit - 2n) {
      return { username, usernameHash };
    }
  }

  throw new Error("Unable to find a username hash within the non-inclusion circuit range");
}

function hashUsername(poseidon: Poseidon, username: bigint[]): bigint {
  const levelOne = Array.from({ length: 8 }, (_, chunkIndex) =>
    poseidon.F.toObject(
      poseidon([
        username[chunkIndex * 4],
        username[chunkIndex * 4 + 1],
        username[chunkIndex * 4 + 2],
        username[chunkIndex * 4 + 3],
      ]),
    ),
  );

  const levelTwo = Array.from({ length: 2 }, (_, chunkIndex) =>
    poseidon.F.toObject(
      poseidon([
        levelOne[chunkIndex * 4],
        levelOne[chunkIndex * 4 + 1],
        levelOne[chunkIndex * 4 + 2],
        levelOne[chunkIndex * 4 + 3],
      ]),
    ),
  );

  return poseidon.F.toObject(poseidon(levelTwo));
}

function toSignalString(value: bigint): string {
  return value.toString();
}
