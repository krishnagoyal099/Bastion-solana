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
exports.BASTION_AMM_PROGRAM_ID = exports.BASTION_POOL_PROGRAM_ID = void 0;
exports.getPoolConfigPda = getPoolConfigPda;
exports.getUserDepositPda = getUserDepositPda;
exports.getOrderPda = getOrderPda;
exports.getNullifierPda = getNullifierPda;
exports.getAmmPoolPda = getAmmPoolPda;
exports.computeCommitmentHash = computeCommitmentHash;
const web3_js_1 = require("@solana/web3.js");
const crypto = __importStar(require("crypto"));
exports.BASTION_POOL_PROGRAM_ID = new web3_js_1.PublicKey('BASTi0N11111111111111111111111111111111111111');
exports.BASTION_AMM_PROGRAM_ID = new web3_js_1.PublicKey('BASTAMM1111111111111111111111111111111111111');
function getPoolConfigPda() {
    return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from('config')], exports.BASTION_POOL_PROGRAM_ID);
}
function getUserDepositPda(user) {
    return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from('deposit'), user.toBuffer()], exports.BASTION_POOL_PROGRAM_ID);
}
function getOrderPda(commitment) {
    return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from('order'), commitment], exports.BASTION_POOL_PROGRAM_ID);
}
function getNullifierPda(nullifierHash) {
    return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from('nullifier'), nullifierHash], exports.BASTION_POOL_PROGRAM_ID);
}
function getAmmPoolPda(mintA, mintB) {
    return web3_js_1.PublicKey.findProgramAddressSync([Buffer.from('pool'), mintA.toBuffer(), mintB.toBuffer()], exports.BASTION_AMM_PROGRAM_ID);
}
function computeCommitmentHash(amount, side, price, nonce) {
    const hash = crypto.createHash('sha256');
    hash.update(Buffer.from(amount.toString()));
    hash.update(Buffer.from(side.toString()));
    hash.update(Buffer.from(price.toString()));
    hash.update(nonce);
    return hash.digest();
}
