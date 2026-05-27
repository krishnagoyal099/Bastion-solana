import { Connection, PublicKey } from '@solana/web3.js';
export interface DarkPoolConfig {
    connection: Connection;
    relayerUrl: string;
    relayerPubKey: Buffer;
}
export declare class DarkPoolClient {
    private connection;
    private relayer;
    private relayerKey;
    constructor(config: DarkPoolConfig);
    submitOrder(params: {
        beneficiary: PublicKey;
        side: 'buy' | 'sell';
        amount: number;
        price: number;
        tokenMint: PublicKey;
    }): Promise<{
        commitment: Buffer<ArrayBufferLike>;
        result: any;
    }>;
}
