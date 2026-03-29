/**
 * UsernameHasher provides Poseidon hashing functionality that matches the
 * username_hash_impl.circom circuit output.
 */
export declare class UsernameHasher {
    private poseidon;
    /**
     * Creates a new UsernameHasher instance.
     * Must be initialized with await UsernameHasher.create()
     */
    private constructor();
    /**
     * Creates and initializes a UsernameHasher instance.
     * @returns Promise resolving to a UsernameHasher instance
     */
    static create(): Promise<UsernameHasher>;
    /**
     * Hashes a username string to a bigint using the same algorithm as the circuit.
     * The username is encoded as ASCII values in a 32-element array, zero-padded.
     * @param username - The username string to hash (max 32 characters)
     * @returns The Poseidon hash as a bigint
     */
    hash(username: string): bigint;
    /**
     * Hashes a pre-encoded username array using the circuit's Poseidon algorithm.
     * @param username - Array of 32 numbers representing ASCII values
     * @returns The Poseidon hash as a bigint
     */
    hashRaw(username: number[]): bigint;
    /**
     * Encodes a username string to a 32-element array of ASCII values.
     * @param username - The username string
     * @returns Array of 32 numbers
     */
    private encodeUsername;
}
