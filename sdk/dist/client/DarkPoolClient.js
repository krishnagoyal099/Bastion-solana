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
exports.DarkPoolClient = void 0;
const RelayerClient_1 = require("../relayer/RelayerClient");
const pda_1 = require("../utils/pda");
const encryption_1 = require("../crypto/encryption");
const crypto = __importStar(require("crypto"));
let wasmProver;
try {
    wasmProver = require('bastion-prover-wasm');
}
catch (e) { }
class DarkPoolClient {
    constructor(config) {
        this.connection = config.connection;
        this.relayer = new RelayerClient_1.RelayerClient(config.relayerUrl);
        this.relayerKey = config.relayerPubKey;
    }
    async submitOrder(params) {
        if (!wasmProver) {
            throw new Error("WASM prover not loaded");
        }
        const sideNum = params.side === 'buy' ? 0 : 1;
        const nonce = crypto.randomBytes(32);
        const commitment = (0, pda_1.computeCommitmentHash)(params.amount, sideNum, params.price, nonce);
        const prover = new wasmProver.ZkProver(10);
        const proofBytesArray = prover.generate_proof(params.amount, sideNum, params.price, nonce);
        const proofBytes = Buffer.from(proofBytesArray);
        const proofHash = crypto.createHash('sha256').update(proofBytes).digest();
        const { encrypted, aesNonce } = (0, encryption_1.encryptOrderDetails)(params.amount, sideNum, params.price, nonce, this.relayerKey);
        const result = await this.relayer.submitOrder({
            commitment: Array.from(commitment),
            beneficiary: params.beneficiary.toBase58(),
            proof_bytes: Array.from(proofBytes),
            proof_hash: Array.from(proofHash),
            encrypted_details: Array.from(encrypted),
            aes_nonce: Array.from(aesNonce)
        });
        return { commitment, result };
    }
}
exports.DarkPoolClient = DarkPoolClient;
