import { Keypair } from "@stellar/stellar-sdk";
import type { AddStellarAddressParams, BuiltTransaction, Bytes32Input, RegisterParams, RegisterResolverParams, StellarTxBuilderConfig, SubmitTransactionOptions, TxBuildOptions } from "./types";
export declare class StellarTxBuilder {
    private readonly config;
    private readonly server;
    private readonly contract;
    constructor(config: StellarTxBuilderConfig);
    buildRegister(params: RegisterParams): Promise<BuiltTransaction>;
    buildRegisterResolver(params: RegisterResolverParams): Promise<BuiltTransaction>;
    buildAddStellarAddress(params: AddStellarAddressParams): Promise<BuiltTransaction>;
    buildResolve(usernameHash: Bytes32Input, options?: TxBuildOptions): Promise<BuiltTransaction>;
    submitTransaction(built: BuiltTransaction | string, signer: Keypair | string, _options?: SubmitTransactionOptions): Promise<any>;
    private buildPreparedTransaction;
}
