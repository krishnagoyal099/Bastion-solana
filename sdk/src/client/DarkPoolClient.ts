import { Connection, PublicKey } from '@solana/web3.js';
import { RelayerClient } from '../relayer/RelayerClient';
import { computeCommitmentHash } from '../utils/pda';
import { encryptOrderDetails } from '../crypto/encryption';
import * as crypto from 'crypto';

let wasmProver: any;
try {
  wasmProver = require('bastion-prover-wasm');
} catch(e) {}

export interface DarkPoolConfig {
  connection: Connection;
  relayerUrl: string;
  relayerPubKey: Buffer; 
}

export class DarkPoolClient {
  private connection: Connection;
  private relayer: RelayerClient;
  private relayerKey: Buffer;

  constructor(config: DarkPoolConfig) {
    this.connection = config.connection;
    this.relayer = new RelayerClient(config.relayerUrl);
    this.relayerKey = config.relayerPubKey;
  }

  async submitOrder(params: {
    beneficiary: PublicKey;
    side: 'buy' | 'sell';
    amount: number;
    price: number;
    tokenMint: PublicKey;
  }) {
    if (!wasmProver) {
      throw new Error("WASM prover not loaded");
    }

    const sideNum = params.side === 'buy' ? 0 : 1;
    const nonce = crypto.randomBytes(32);
    
    const commitment = computeCommitmentHash(params.amount, sideNum, params.price, nonce);
    
    const prover = new wasmProver.ZkProver(10);
    const proofBytesArray = prover.generate_proof(params.amount, sideNum, params.price, nonce);
    const proofBytes = Buffer.from(proofBytesArray);
    
    const proofHash = crypto.createHash('sha256').update(proofBytes).digest();
    
    const { encrypted, aesNonce } = encryptOrderDetails(
      params.amount,
      sideNum,
      params.price,
      nonce,
      this.relayerKey
    );

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
