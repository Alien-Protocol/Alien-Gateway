"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NoAddressLinkedError = exports.UsernameNotFoundError = exports.TransactionFailedError = exports.ProofGenerationError = exports.UsernameUnavailableError = exports.AlienGatewayError = void 0;
class AlienGatewayError extends Error {
    constructor(message, options) {
        super(message);
        this.name = new.target.name;
        this.cause = options?.cause;
    }
}
exports.AlienGatewayError = AlienGatewayError;
class UsernameUnavailableError extends AlienGatewayError {
    constructor(username, commitment) {
        super(`Username "${username}" is already registered.`);
        this.username = username;
        this.commitment = commitment;
    }
}
exports.UsernameUnavailableError = UsernameUnavailableError;
class ProofGenerationError extends AlienGatewayError {
    constructor(message, options) {
        super(message, options);
    }
}
exports.ProofGenerationError = ProofGenerationError;
class TransactionFailedError extends AlienGatewayError {
    constructor(message, txHash, options) {
        super(message, options);
        this.txHash = txHash;
    }
}
exports.TransactionFailedError = TransactionFailedError;
class UsernameNotFoundError extends AlienGatewayError {
    constructor(username) {
        super(`Username "${username}" not found.`);
        this.username = username;
    }
}
exports.UsernameNotFoundError = UsernameNotFoundError;
class NoAddressLinkedError extends AlienGatewayError {
    constructor(username) {
        super(`Username "${username}" does not have a linked Stellar address.`);
        this.username = username;
    }
}
exports.NoAddressLinkedError = NoAddressLinkedError;
