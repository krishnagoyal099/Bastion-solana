import { Connection } from '@solana/web3.js';
import { DarkPoolClient } from './DarkPoolClient';

export interface BastionClientConfig {
  rpcUrl: string;
  wsUrl?: string;
  commitment?: 'processed' | 'confirmed' | 'finalized';
  relayerUrl: string;
  relayerPubKey: Buffer;
}

export class BastionClient {
  public connection: Connection;
  public darkPool: DarkPoolClient;

  constructor(config: BastionClientConfig) {
    this.connection = new Connection(config.rpcUrl, config.commitment || 'confirmed');
    
    this.darkPool = new DarkPoolClient({
      connection: this.connection,
      relayerUrl: config.relayerUrl,
      relayerPubKey: config.relayerPubKey
    });
  }
}
