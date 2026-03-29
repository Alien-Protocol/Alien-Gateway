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
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerUsername = exports.UsernameUnavailableError = exports.UsernameNotFoundError = exports.TransactionFailedError = exports.ProofGenerationError = exports.NoAddressLinkedError = exports.AlienGatewayError = exports.UsernameResolver = exports.hashUsername = exports.encodeUsername = exports.bigintToBytes32 = exports.UsernameHasher = exports.autoDetectWallet = exports.WalletDetectionError = exports.XBullAdapter = exports.FreighterAdapter = exports.MerkleProofGenerator = void 0;
var proof_1 = require("./proof");
Object.defineProperty(exports, "MerkleProofGenerator", { enumerable: true, get: function () { return proof_1.MerkleProofGenerator; } });
var wallets_1 = require("./wallets");
Object.defineProperty(exports, "FreighterAdapter", { enumerable: true, get: function () { return wallets_1.FreighterAdapter; } });
Object.defineProperty(exports, "XBullAdapter", { enumerable: true, get: function () { return wallets_1.XBullAdapter; } });
Object.defineProperty(exports, "WalletDetectionError", { enumerable: true, get: function () { return wallets_1.WalletDetectionError; } });
Object.defineProperty(exports, "autoDetectWallet", { enumerable: true, get: function () { return wallets_1.autoDetectWallet; } });
var hasher_1 = require("./hasher");
Object.defineProperty(exports, "UsernameHasher", { enumerable: true, get: function () { return hasher_1.UsernameHasher; } });
var hash_1 = require("./hash");
Object.defineProperty(exports, "bigintToBytes32", { enumerable: true, get: function () { return hash_1.bigintToBytes32; } });
Object.defineProperty(exports, "encodeUsername", { enumerable: true, get: function () { return hash_1.encodeUsername; } });
Object.defineProperty(exports, "hashUsername", { enumerable: true, get: function () { return hash_1.hashUsername; } });
var resolver_1 = require("./resolver");
Object.defineProperty(exports, "UsernameResolver", { enumerable: true, get: function () { return resolver_1.UsernameResolver; } });
var errors_1 = require("./errors");
Object.defineProperty(exports, "AlienGatewayError", { enumerable: true, get: function () { return errors_1.AlienGatewayError; } });
Object.defineProperty(exports, "NoAddressLinkedError", { enumerable: true, get: function () { return errors_1.NoAddressLinkedError; } });
Object.defineProperty(exports, "ProofGenerationError", { enumerable: true, get: function () { return errors_1.ProofGenerationError; } });
Object.defineProperty(exports, "TransactionFailedError", { enumerable: true, get: function () { return errors_1.TransactionFailedError; } });
Object.defineProperty(exports, "UsernameNotFoundError", { enumerable: true, get: function () { return errors_1.UsernameNotFoundError; } });
Object.defineProperty(exports, "UsernameUnavailableError", { enumerable: true, get: function () { return errors_1.UsernameUnavailableError; } });
var register_1 = require("./register");
Object.defineProperty(exports, "registerUsername", { enumerable: true, get: function () { return register_1.registerUsername; } });
__exportStar(require("./availability"), exports);
