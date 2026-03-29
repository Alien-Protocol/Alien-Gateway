export interface WalletAdapter {
    connect(): Promise<void>;
    getPublicKey(): Promise<string>;
    signTransaction(xdr: string): Promise<string>;
}
export declare class WalletDetectionError extends Error {
    constructor(message: string);
}
export declare function assertBrowserEnvironment(): void;
export declare function getBrowserWindow<T extends object>(): T;
