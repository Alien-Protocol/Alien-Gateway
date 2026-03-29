"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.StellarTxBuilder = void 0;
const stellar_sdk_1 = require("@stellar/stellar-sdk");
const DEFAULT_TIMEOUT_SECONDS = 60;
class StellarTxBuilder {
    constructor(config) {
        this.config = config;
        this.server = new stellar_sdk_1.SorobanRpc.Server(config.rpcUrl, {
            allowHttp: config.allowHttp ?? isHttpUrl(config.rpcUrl),
        });
        this.contract = new stellar_sdk_1.Contract(config.contractAddress);
    }
    async buildRegister(params) {
        return this.buildPreparedTransaction("register", [
            toScAddress(params.caller),
            toScBytes32(params.commitment),
        ], params);
    }
    async buildRegisterResolver(params) {
        return this.buildPreparedTransaction("register_resolver", [
            toScAddress(params.caller),
            toScBytes32(params.commitment),
            toScBytes(params.proof),
            toScPublicSignals(params.publicSignals),
        ], params);
    }
    async buildAddStellarAddress(params) {
        return this.buildPreparedTransaction("add_stellar_address", [
            toScAddress(params.caller),
            toScBytes32(params.usernameHash),
            toScAddress(params.stellarAddress),
        ], params);
    }
    async buildResolve(usernameHash, options = {}) {
        return this.buildPreparedTransaction("resolve_stellar", [toScBytes32(usernameHash)], options);
    }
    async submitTransaction(built, signer, _options = {}) {
        const signed = typeof built === "string"
            ? stellar_sdk_1.TransactionBuilder.fromXDR(built, this.config.networkPassphrase)
            : stellar_sdk_1.TransactionBuilder.fromXDR(built.xdr, this.config.networkPassphrase);
        const keypair = typeof signer === "string" ? stellar_sdk_1.Keypair.fromSecret(signer) : signer;
        signed.sign(keypair);
        return this.server.sendTransaction(signed);
    }
    async buildPreparedTransaction(method, args, options) {
        const source = resolveSource(options, this.config.defaultSource);
        if (!source) {
            throw new Error(`A source account is required to build ${method} transactions`);
        }
        const account = await this.server.getAccount(source);
        const raw = new stellar_sdk_1.TransactionBuilder(account, {
            fee: options.fee ?? this.config.defaultFee ?? stellar_sdk_1.BASE_FEE,
            networkPassphrase: this.config.networkPassphrase,
        })
            .addOperation(this.contract.call(method, ...args))
            .setTimeout(options.timeoutInSeconds ?? this.config.timeoutInSeconds ?? DEFAULT_TIMEOUT_SECONDS)
            .build();
        const prepared = await this.server.prepareTransaction(raw);
        return {
            transaction: prepared,
            xdr: prepared.toXDR(),
            method,
            source,
        };
    }
}
exports.StellarTxBuilder = StellarTxBuilder;
function toScAddress(address) {
    return new stellar_sdk_1.Address(address).toScVal();
}
function toScBytes32(value) {
    const bytes = normalizeBytes(value);
    if (bytes.length !== 32) {
        throw new Error(`Expected 32 bytes, received ${bytes.length}`);
    }
    return (0, stellar_sdk_1.nativeToScVal)(bytes, { type: "bytes" });
}
function toScBytes(value) {
    return (0, stellar_sdk_1.nativeToScVal)(normalizeBytes(value), { type: "bytes" });
}
function toScPublicSignals(value) {
    return (0, stellar_sdk_1.nativeToScVal)({
        old_root: normalizeBytes32(value.oldRoot),
        new_root: normalizeBytes32(value.newRoot),
    });
}
function normalizeBytes32(value) {
    const bytes = normalizeBytes(value);
    if (bytes.length !== 32) {
        throw new Error(`Expected 32 bytes, received ${bytes.length}`);
    }
    return bytes;
}
function normalizeBytes(value) {
    if (typeof value !== "string") {
        return Buffer.from(value);
    }
    const normalized = value.startsWith("0x") ? value.slice(2) : value;
    if (normalized.length % 2 === 0 && /^[0-9a-fA-F]+$/.test(normalized)) {
        return Buffer.from(normalized, "hex");
    }
    return Buffer.from(value, "base64");
}
function isHttpUrl(url) {
    return url.startsWith("http://");
}
function resolveSource(options, defaultSource) {
    return options.source ?? options.caller ?? defaultSource;
}
