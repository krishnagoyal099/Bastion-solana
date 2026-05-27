export declare function encryptOrderDetails(amount: number, side: number, price: number, nonce: Buffer, relayerKey: Buffer): {
    encrypted: Buffer;
    aesNonce: Buffer;
};
