import * as fs from "fs";
import * as path from "path";
import snarkjs from "snarkjs";

/**
 * Minimal proof generator for `merkle_non_inclusion`.
 *
 * Usage (from `zk/`):
 *   npx ts-node scripts/prove_non_inclusion.ts ./input.json
 *
 * The `input.json` must match the circuit's signal names, e.g.:
 * {
 *   "username": ["... 32 elements ..."],
 *   "leaf_before": "...",
 *   "leaf_after": "...",
 *   "merklePathBeforeSiblings": ["... 20 ..."],
 *   "merklePathBeforeIndices": ["... 20 ..."],
 *   "merklePathAfterSiblings": ["... 20 ..."],
 *   "merklePathAfterIndices": ["... 20 ..."],
 *   "root": "..."
 * }
 */
async function main() {
  const inputPath = process.argv[2];
  if (!inputPath) {
    throw new Error("Missing input path. Usage: prove_non_inclusion.ts <input.json>");
  }

  const CIRCUIT = "merkle_non_inclusion";
  const BUILD_DIR = path.join("build", CIRCUIT);
  const WASM_PATH = path.join(BUILD_DIR, "wasm", `${CIRCUIT}_js`, `${CIRCUIT}.wasm`);
  const ZKEY_PATH = path.join(BUILD_DIR, `${CIRCUIT}_final.zkey`);

  const raw = fs.readFileSync(inputPath, "utf8");
  const input = JSON.parse(raw);

  const { proof, publicSignals } = await snarkjs.groth16.fullProve(input, WASM_PATH, ZKEY_PATH);

  process.stdout.write(
    JSON.stringify(
      {
        publicSignals,
        proof,
      },
      null,
      2,
    ),
  );
  process.stdout.write("\n");
}

main().catch((err) => {
  process.stderr.write(String(err) + "\n");
  process.exit(1);
});

