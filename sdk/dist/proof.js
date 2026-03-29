"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MerkleProofGenerator = void 0;
// @ts-ignore - snarkjs doesn't have type definitions
const snarkjs_1 = __importDefault(require("snarkjs"));
class MerkleProofGenerator {
    constructor(config) {
        this.config = config;
    }
    async proveInclusion(input) {
        // @ts-ignore - snarkjs types are not fully compatible with our input types
        const { proof, publicSignals } = await snarkjs_1.default.groth16.fullProve(
        // @ts-ignore
        normalizeInput(input), this.config.inclusion.wasmPath, this.config.inclusion.zkeyPath);
        return {
            proof: proof,
            publicSignals: publicSignals,
        };
    }
    async proveNonInclusion(input) {
        // @ts-ignore - snarkjs types are not fully compatible with our input types
        const { proof, publicSignals } = await snarkjs_1.default.groth16.fullProve(
        // @ts-ignore
        normalizeInput(input), this.config.nonInclusion.wasmPath, this.config.nonInclusion.zkeyPath);
        return {
            proof: proof,
            publicSignals: publicSignals,
        };
    }
}
exports.MerkleProofGenerator = MerkleProofGenerator;
function normalizeInput(input) {
    return Object.fromEntries(Object.entries(input).map(([key, value]) => [key, normalizeSignal(value)]));
}
function normalizeSignal(value) {
    if (Array.isArray(value)) {
        return value.map((entry) => normalizeSignal(entry));
    }
    return typeof value === "bigint" ? value.toString() : String(value);
}
