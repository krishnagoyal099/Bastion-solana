import { PublicKey } from '@solana/web3.js';
import * as crypto from 'crypto';

export const BASTION_POOL_PROGRAM_ID = new PublicKey('BASTi0N11111111111111111111111111111111111111');
export const BASTION_AMM_PROGRAM_ID = new PublicKey('BASTAMM1111111111111111111111111111111111111');

export function getPoolConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getUserDepositPda(user: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('deposit'), user.toBuffer()],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getOrderPda(commitment: Buffer): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('order'), commitment],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getNullifierPda(nullifierHash: Buffer): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), nullifierHash],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getEscrowPda(commitment: Buffer): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('escrow'), commitment],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getTokenVaultPda(config: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('token_vault'), config.toBuffer()],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getWsolVaultPda(config: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('wsol_vault'), config.toBuffer()],
    BASTION_POOL_PROGRAM_ID
  );
}

export function getAmmPoolPda(mintA: PublicKey, mintB: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('pool'), mintA.toBuffer(), mintB.toBuffer()],
    BASTION_AMM_PROGRAM_ID
  );
}

/**
 * Compute commitment hash matching on-chain reveal_order verification.
 * Format: sha256(amount_le_u64 || side_u8 || price_le_u64 || nonce_32bytes)
 * 
 * MUST match the hashv() call in reveal_order.rs:
 *   hashv(&[&amount.to_le_bytes(), &[side], &price.to_le_bytes(), &nonce])
 */
export function computeCommitmentHash(
  amount: bigint,
  side: number,
  price: bigint,
  nonce: Buffer
): Buffer {
  const amountBuf = Buffer.alloc(8);
  amountBuf.writeBigUInt64LE(amount);

  const sideBuf = Buffer.from([side & 0xFF]);

  const priceBuf = Buffer.alloc(8);
  priceBuf.writeBigUInt64LE(price);

  const hash = crypto.createHash('sha256');
  hash.update(amountBuf);
  hash.update(sideBuf);
  hash.update(priceBuf);
  hash.update(nonce);
  return hash.digest();
}

