import type * as FreighterApiModule from "@stellar/freighter-api";
import type { WalletAdapter } from "./shared";
export type FreighterApi = Pick<typeof FreighterApiModule, "requestAccess" | "getPublicKey" | "getAddress" | "signTransaction">;
export declare class FreighterAdapter implements WalletAdapter {
    private readonly api?;
    private publicKey?;
    constructor(api?: FreighterApi | undefined);
    static isAvailable(): Promise<boolean>;
    connect(): Promise<void>;
    getPublicKey(): Promise<string>;
    signTransaction(xdr: string): Promise<string>;
    private getApi;
}
