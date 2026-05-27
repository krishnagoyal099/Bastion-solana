import { Connection } from '@solana/web3.js';
import { DarkPoolClient } from './DarkPoolClient';
export interface BastionClientConfig {
    rpcUrl: string;
    wsUrl?: string;
    commitment?: 'processed' | 'confirmed' | 'finalized';
    relayerUrl: string;
    relayerPubKey: Buffer;
}
export declare class BastionClient {
    connection: Connection;
    darkPool: DarkPoolClient;
    constructor(config: BastionClientConfig);
}
