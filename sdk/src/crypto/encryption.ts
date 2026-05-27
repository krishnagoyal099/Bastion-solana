import * as crypto from 'crypto';

export function encryptOrderDetails(
  amount: number,
  side: number,
  price: number,
  nonce: Buffer,
  relayerKey: Buffer
): { encrypted: Buffer, aesNonce: Buffer } {
  const payload = JSON.stringify({
    amount,
    side,
    price,
    nonce: Array.from(nonce)
  });
  
  const aesNonce = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', relayerKey, aesNonce);
  
  let encrypted = cipher.update(payload, 'utf8');
  encrypted = Buffer.concat([encrypted, cipher.final()]);
  const authTag = cipher.getAuthTag();
  
  const fullEncrypted = Buffer.concat([encrypted, authTag]);
  
  return {
    encrypted: fullEncrypted,
    aesNonce
  };
}
