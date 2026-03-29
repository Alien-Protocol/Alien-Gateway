"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WalletDetectionError = void 0;
exports.assertBrowserEnvironment = assertBrowserEnvironment;
exports.getBrowserWindow = getBrowserWindow;
class WalletDetectionError extends Error {
    constructor(message) {
        super(message);
        this.name = "WalletDetectionError";
    }
}
exports.WalletDetectionError = WalletDetectionError;
function assertBrowserEnvironment() {
    if (typeof globalThis.window === "undefined") {
        throw new WalletDetectionError("Wallet adapters are only available in browser environments.");
    }
}
function getBrowserWindow() {
    assertBrowserEnvironment();
    return globalThis.window;
}
