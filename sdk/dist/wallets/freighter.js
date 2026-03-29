"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.FreighterAdapter = void 0;
const shared_1 = require("./shared");
class FreighterAdapter {
    constructor(api) {
        this.api = api;
    }
    static async isAvailable() {
        if (typeof globalThis.window === "undefined") {
            return false;
        }
        try {
            // @ts-ignore - @stellar/freighter-api is an optional dependency
            await Promise.resolve().then(() => __importStar(require("@stellar/freighter-api")));
            return true;
        }
        catch {
            return false;
        }
    }
    async connect() {
        const freighter = await this.getApi();
        const response = await freighter.requestAccess();
        const publicKey = extractFreighterString(response, "Freighter access request failed.");
        if (!publicKey) {
            throw new shared_1.WalletDetectionError("Freighter did not return a public key.");
        }
        this.publicKey = publicKey;
    }
    async getPublicKey() {
        if (this.publicKey) {
            return this.publicKey;
        }
        const freighter = await this.getApi();
        const addressResult = freighter.getPublicKey
            ? await freighter.getPublicKey()
            : freighter.getAddress
                ? await freighter.getAddress()
                : undefined;
        const publicKey = extractFreighterString(addressResult, "Unable to read the active Freighter public key.");
        if (!publicKey) {
            throw new shared_1.WalletDetectionError("Freighter did not expose an active public key.");
        }
        this.publicKey = publicKey;
        return publicKey;
    }
    async signTransaction(xdr) {
        const freighter = await this.getApi();
        const signed = await freighter.signTransaction(xdr);
        return extractSignedXdr(signed, "Freighter failed to sign the Soroban transaction.");
    }
    async getApi() {
        if (this.api) {
            return this.api;
        }
        (0, shared_1.assertBrowserEnvironment)();
        try {
            // @ts-ignore - @stellar/freighter-api is an optional dependency
            return await Promise.resolve().then(() => __importStar(require("@stellar/freighter-api")));
        }
        catch (error) {
            throw new shared_1.WalletDetectionError("Freighter is not available in this browser.");
        }
    }
}
exports.FreighterAdapter = FreighterAdapter;
function extractFreighterString(value, fallbackMessage) {
    if (typeof value === "string") {
        return value;
    }
    if (value?.error) {
        throw new shared_1.WalletDetectionError(value.error);
    }
    const address = "address" in (value ?? {}) ? value.address : undefined;
    const publicKey = "publicKey" in (value ?? {}) ? value.publicKey : undefined;
    const result = address ?? publicKey;
    if (!result) {
        throw new shared_1.WalletDetectionError(fallbackMessage);
    }
    return result;
}
function extractSignedXdr(value, fallbackMessage) {
    if (typeof value === "string") {
        return value;
    }
    if (value.error) {
        throw new shared_1.WalletDetectionError(value.error);
    }
    const signedXdr = value.signedTxXdr ?? value.signedXDR ?? value.xdr;
    if (!signedXdr) {
        throw new shared_1.WalletDetectionError(fallbackMessage);
    }
    return signedXdr;
}
