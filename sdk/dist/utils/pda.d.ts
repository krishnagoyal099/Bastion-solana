import { PublicKey } from '@solana/web3.js';
export declare const BASTION_POOL_PROGRAM_ID: PublicKey;
export declare const BASTION_AMM_PROGRAM_ID: PublicKey;
export declare function getPoolConfigPda(): [PublicKey, number];
export declare function getUserDepositPda(user: PublicKey): [PublicKey, number];
export declare function getOrderPda(commitment: Buffer): [PublicKey, number];
export declare function getNullifierPda(nullifierHash: Buffer): [PublicKey, number];
export declare function getAmmPoolPda(mintA: PublicKey, mintB: PublicKey): [PublicKey, number];
export declare function computeCommitmentHash(amount: number, side: number, price: number, nonce: Buffer): Buffer;
