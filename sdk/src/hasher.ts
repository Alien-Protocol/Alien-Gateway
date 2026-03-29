import { buildPoseidon } from "circomlibjs";

/**
 * UsernameHasher provides Poseidon hashing functionality that matches the
 * username_hash_impl.circom circuit output.
 */
export class UsernameHasher {
  private poseidon: any;

  /**
   * Creates a new UsernameHasher instance.
   * Must be initialized with await UsernameHasher.create()
   */
  private constructor(poseidon: any) {
    this.poseidon = poseidon;
  }

  /**
   * Creates and initializes a UsernameHasher instance.
   * @returns Promise resolving to a UsernameHasher instance
   */
  static async create(): Promise<UsernameHasher> {
    const poseidon = await buildPoseidon();
    return new UsernameHasher(poseidon);
  }

  /**
   * Hashes a username string to a bigint using the same algorithm as the circuit.
   * The username is encoded as ASCII values in a 32-element array, zero-padded.
   * @param username - The username string to hash (max 32 characters)
   * @returns The Poseidon hash as a bigint
   */
  hash(username: string): bigint {
    if (username.length > 32) {
      throw new Error("Username must be 32 characters or less");
    }

    const encoded = this.encodeUsername(username);
    return this.hashRaw(encoded);
  }

  /**
   * Hashes a pre-encoded username array using the circuit's Poseidon algorithm.
   * @param username - Array of 32 numbers representing ASCII values
   * @returns The Poseidon hash as a bigint
   */
  hashRaw(username: number[]): bigint {
    if (username.length !== 32) {
      throw new Error("Username array must contain exactly 32 elements");
    }

    const F = this.poseidon.F;

    // Step 1: Hash in chunks of 4 (8 chunks)
    const h: bigint[] = [];
    for (let i = 0; i < 8; i++) {
      h[i] = F.toObject(
        this.poseidon([
          BigInt(username[i * 4 + 0]),
          BigInt(username[i * 4 + 1]),
          BigInt(username[i * 4 + 2]),
          BigInt(username[i * 4 + 3]),
        ])
      );
    }

    // Step 2: Hash intermediate hashes in chunks of 4 (2 chunks)
    const h2: bigint[] = [];
    for (let i = 0; i < 2; i++) {
      const j = i * 4;
      h2[i] = F.toObject(
        this.poseidon([h[j + 0], h[j + 1], h[j + 2], h[j + 3]])
      );
    }

    // Step 3: Final hash
    return F.toObject(this.poseidon([h2[0], h2[1]]));
  }

  /**
   * Encodes a username string to a 32-element array of ASCII values.
   * @param username - The username string
   * @returns Array of 32 numbers
   */
  private encodeUsername(username: string): number[] {
    const encoded = new Array(32).fill(0);
    for (let i = 0; i < Math.min(username.length, 32); i++) {
      const charCode = username.charCodeAt(i);
      if (charCode > 127) {
        throw new Error("Username contains non-ASCII characters");
      }
      encoded[i] = charCode;
    }
    return encoded;
  }
}