/**
 * Configuration for the Stellar network and core contract used by the resolver.
 */
export interface NetworkConfig {
    /** The Stellar network to use ("testnet" or "mainnet"). */
    network: "testnet" | "mainnet";
    /** The Soroban RPC URL for querying the contract. */
    rpcUrl: string;
    /** The core contract ID on the specified network. */
    contractId: string;
}
/**
 * Result of a resolveWithMemo call, containing both the address and optional memo.
 */
export interface ResolveWithMemoResult {
    /** The linked Stellar address. */
    address: string;
    /** The optional memo (if any) linked to the username. */
    memo?: string;
}
/**
 * UsernameResolver is the core user-facing SDK component for resolving
 * human-readable usernames to their linked Stellar addresses.
 */
export declare class UsernameResolver {
    private readonly config;
    /**
     * Creates a new instance of the UsernameResolver.
     *
     * @param config - The network configuration including RPC URL and contract ID.
     */
    constructor(config: NetworkConfig);
    /**
     * Resolves a username string to its linked Stellar address.
     *
     * @param username - A username string (e.g., "alice").
     * @returns A promise that resolves to the linked Stellar address.
     * @throws {@link UsernameNotFoundError} if the username is not registered on the gateway.
     * @throws {@link NoAddressLinkedError} if the username is registered but has no linked Stellar address.
     */
    resolve(username: string): Promise<string>;
    /**
     * Resolves a username string to its linked Stellar address and optional memo.
     *
     * @param username - A username string (e.g., "alice").
     * @returns A promise that resolves to an object containing the address and memo.
     * @throws {@link UsernameNotFoundError} if the username is not registered.
     * @throws {@link NoAddressLinkedError} if no address is linked to the registered username.
     */
    resolveWithMemo(username: string): Promise<ResolveWithMemoResult>;
    /**
     * Internal helper to perform the RPC call.
     * In tests, this method or the underlying fetch is mocked.
     */
    private fetchFromRpc;
}
