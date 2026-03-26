import * as path from "path";
import * as assert from "assert";
import * as fs from "fs";
import { snarkjs } from "snarkjs";
import { buildPoseidon } from "circomlibjs";

// ── Paths ────────────────────────────────────────────────────────────────────

const CIRCUIT = "username_leaf_main";
const BUILD_DIR = path.join(__dirname, "..", "build", CIRCUIT);
const WASM_PATH = path.join(BUILD_DIR, "wasm", `${CIRCUIT}_js`, `${CIRCUIT}.wasm`);
const INPUT_PATH = path.join(__dirname, "..", "input.json");

// ── Username Encoding Documentation ────────────────────────────────────────────

/**
 * Username Encoding Format:
 * 
 * Usernames are encoded as 32-byte arrays using ASCII character values,
 * with zero-padding for unused bytes.
 * 
 * Example: "amar" becomes:
 * [97, 109, 97, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 *  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
 * 
 * Where:
 * - 97 = 'a', 109 = 'm', 97 = 'a', 114 = 'r'
 * - Remaining 28 bytes are zeros (padding)
 * 
 * This format matches the encoding used in zk/input.json and is
 * consistent across the Alien Gateway ZK circuits.
 */

// ── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Convert a string username to 32-byte array format
 * @param username - The username string (max 32 characters)
 * @returns 32-element array of ASCII values with zero padding
 */
function usernameToBytes(username: string): number[] {
    const bytes = new Array(32).fill(0);
    for (let i = 0; i < Math.min(username.length, 32); i++) {
        bytes[i] = username.charCodeAt(i);
    }
    return bytes;
}

/**
 * Compute Poseidon hash of username using TypeScript implementation
 * This should match the circuit output exactly
 */
async function computeUsernameHashTS(username: string): Promise<bigint> {
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    
    // Convert username to 32-byte array
    const usernameBytes = usernameToBytes(username);
    
    // Step 1: Hash in chunks of 4 (same as circuit)
    const h: bigint[] = [];
    for (let i = 0; i < 8; i++) {
        const chunk = [
            BigInt(usernameBytes[i*4]),
            BigInt(usernameBytes[i*4 + 1]),
            BigInt(usernameBytes[i*4 + 2]),
            BigInt(usernameBytes[i*4 + 3])
        ];
        h.push(F.toObject(poseidon(chunk)));
    }
    
    // Step 2: Hash intermediate hashes
    const h2: bigint[] = [];
    for (let i = 0; i < 2; i++) {
        const chunk = [h[i*4], h[i*4 + 1], h[i*4 + 2], h[i*4 + 3]];
        h2.push(F.toObject(poseidon(chunk)));
    }
    
    // Final hash
    const finalHash = F.toObject(poseidon([h2[0], h2[1]]));
    return finalHash;
}

// ── Test runner ──────────────────────────────────────────────────────────────

async function runTests() {
    console.log("\n── Username Leaf Circuit Test ────────────────────────────────\n");
    
    const testUsername = "amar";
    console.log(`Testing username: "${testUsername}"`);
    
    // Load circuit input
    const input = JSON.parse(fs.readFileSync(INPUT_PATH, 'utf8'));
    console.log("✓ Loaded input from input.json");
    
    // Compute expected hash using TypeScript implementation
    const expectedHash = await computeUsernameHashTS(testUsername);
    console.log(`✓ TypeScript Poseidon hash: ${expectedHash.toString()}`);
    
    // Generate witness using circuit
    console.log("\n── Generating circuit witness ───────────────────────────────");
    
    const wasmBuffer = fs.readFileSync(WASM_PATH);
    const { witness } = await snarkjs.wtns.calculateWitnessFromBuffer(wasmBuffer, input);
    console.log("✓ Witness generated successfully");
    
    // Extract leaf output from witness
    // The leaf output should be at index 1 (after the first signal which is usually a dummy)
    const circuitHash = BigInt(witness[1]);
    console.log(`✓ Circuit hash output: ${circuitHash.toString()}`);
    
    // Verify circuit output matches TypeScript computation
    console.log("\n── Verification ─────────────────────────────────────────────");
    
    assert.strictEqual(
        circuitHash.toString(),
        expectedHash.toString(),
        "Circuit hash should match TypeScript Poseidon hash"
    );
    console.log("✓ Circuit output matches TypeScript computation");
    
    // Test with different usernames
    const testCases = ["test", "user123", "alice", ""];
    
    for (const testCase of testCases) {
        console.log(`\n── Testing username: "${testCase}" ────────────────────────`);
        
        const bytes = usernameToBytes(testCase);
        const testCaseInput = { username: bytes };
        
        const testCaseExpected = await computeUsernameHashTS(testCase);
        
        const { witness: testCaseWitness } = await snarkjs.wtns.calculateWitnessFromBuffer(
            wasmBuffer, 
            testCaseInput
        );
        const testCaseCircuitHash = BigInt(testCaseWitness[1]);
        
        assert.strictEqual(
            testCaseCircuitHash.toString(),
            testCaseExpected.toString(),
            `Hash mismatch for username "${testCase}"`
        );
        
        console.log(`✓ Hash: ${testCaseCircuitHash.toString()}`);
    }
    
    console.log("\n══ All tests passed! ══");
    console.log("\n── Username Encoding Summary ────────────────────────────────");
    console.log("Format: 32-byte array, ASCII values, zero-padded");
    console.log("Example 'amar': [97, 109, 97, 114, 0, 0, ..., 0]");
    console.log("Hash algorithm: Poseidon with 4-element chunking");
    console.log("✓ Circuit and TypeScript implementations match perfectly");
}

// ── Run tests ────────────────────────────────────────────────────────────────

runTests().catch(err => {
    console.error(`\n✘ Test failed: ${err.message ?? err}`);
    process.exit(1);
});
