import axios from 'axios';

export interface SubmitOrderPayload {
  commitment: number[];
  beneficiary: string;
  proof_bytes: number[];
  proof_hash: number[];
  encrypted_details: number[];
  aes_nonce: number[];
}

export class RelayerClient {
  constructor(private endpoint: string) {}

  async submitOrder(payload: SubmitOrderPayload): Promise<any> {
    const response = await axios.post(`${this.endpoint}/order`, payload);
    return response.data;
  }
  
  async getHealth(): Promise<boolean> {
    try {
      const response = await axios.get(`${this.endpoint}/health`);
      return response.data === 'OK';
    } catch {
      return false;
    }
  }
}
