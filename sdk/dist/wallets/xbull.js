"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.XBullAdapter = void 0;
const shared_1 = require("./shared");
class XBullAdapter {
    constructor(provider) {
        this.provider = provider;
    }
    static isAvailable() {
        if (typeof globalThis.window === "undefined") {
            return false;
        }
        const browserWindow = (0, shared_1.getBrowserWindow)();
        return Boolean(browserWindow.xBullSDK ?? browserWindow.xBull);
    }
    async connect() {
        const provider = this.getProvider();
        if (provider.connect) {
            await provider.connect();
        }
        else if (provider.requestAccess) {
            await provider.requestAccess();
        }
        this.publicKey = await this.getPublicKey();
    }
    async getPublicKey() {
        if (this.publicKey) {
            return this.publicKey;
        }
        const provider = this.getProvider();
        const publicKey = provider.getPublicKey
            ? await provider.getPublicKey()
            : provider.getAddress
                ? await provider.getAddress()
                : undefined;
        if (!publicKey) {
            throw new shared_1.WalletDetectionError("xBull did not expose an active public key.");
        }
        this.publicKey = publicKey;
        return publicKey;
    }
    async signTransaction(xdr) {
        const provider = this.getProvider();
        const signer = provider.signTransaction ?? provider.sign;
        if (!signer) {
            throw new shared_1.WalletDetectionError("xBull does not support transaction signing.");
        }
        const signed = await signer.call(provider, xdr);
        if (typeof signed === "string") {
            return signed;
        }
        const signedXdr = signed.signedXdr ?? signed.signedTxXdr ?? signed.xdr;
        if (!signedXdr) {
            throw new shared_1.WalletDetectionError("xBull returned an invalid signed transaction payload.");
        }
        return signedXdr;
    }
    getProvider() {
        if (this.provider) {
            return this.provider;
        }
        const browserWindow = (0, shared_1.getBrowserWindow)();
        const provider = browserWindow.xBullSDK ?? browserWindow.xBull;
        if (!provider) {
            throw new shared_1.WalletDetectionError("xBull is not available in this browser.");
        }
        return provider;
    }
}
exports.XBullAdapter = XBullAdapter;
