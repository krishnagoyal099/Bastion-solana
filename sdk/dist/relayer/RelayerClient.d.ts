export interface SubmitOrderPayload {
    commitment: number[];
    beneficiary: string;
    proof_bytes: number[];
    proof_hash: number[];
    encrypted_details: number[];
    aes_nonce: number[];
}
export declare class RelayerClient {
    private endpoint;
    constructor(endpoint: string);
    submitOrder(payload: SubmitOrderPayload): Promise<any>;
    getHealth(): Promise<boolean>;
}
