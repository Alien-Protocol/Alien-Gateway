import type { WalletAdapter } from "./shared";
export interface XBullProvider {
    connect?(): Promise<void>;
    requestAccess?(): Promise<void>;
    getPublicKey?(): Promise<string>;
    getAddress?(): Promise<string>;
    signTransaction?(xdr: string, options?: Record<string, unknown>): Promise<string | {
        signedXdr?: string;
        signedTxXdr?: string;
        xdr?: string;
    }>;
    sign?(xdr: string, options?: Record<string, unknown>): Promise<string | {
        signedXdr?: string;
        signedTxXdr?: string;
        xdr?: string;
    }>;
}
export declare class XBullAdapter implements WalletAdapter {
    private readonly provider?;
    private publicKey?;
    constructor(provider?: XBullProvider | undefined);
    static isAvailable(): boolean;
    connect(): Promise<void>;
    getPublicKey(): Promise<string>;
    signTransaction(xdr: string): Promise<string>;
    private getProvider;
}
