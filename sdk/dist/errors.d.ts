export declare class AlienGatewayError extends Error {
    readonly cause?: unknown;
    constructor(message: string, options?: {
        cause?: unknown;
    });
}
export declare class UsernameUnavailableError extends AlienGatewayError {
    readonly username: string;
    readonly commitment: string;
    constructor(username: string, commitment: string);
}
export declare class ProofGenerationError extends AlienGatewayError {
    constructor(message: string, options?: {
        cause?: unknown;
    });
}
export declare class TransactionFailedError extends AlienGatewayError {
    readonly txHash?: string | undefined;
    constructor(message: string, txHash?: string | undefined, options?: {
        cause?: unknown;
    });
}
export declare class UsernameNotFoundError extends AlienGatewayError {
    readonly username: string;
    constructor(username: string);
}
export declare class NoAddressLinkedError extends AlienGatewayError {
    readonly username: string;
    constructor(username: string);
}
