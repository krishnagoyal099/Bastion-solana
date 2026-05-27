use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key
};
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid key size")]
    InvalidKey,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderDetails {
    pub amount: u64,
    pub side: u8,
    pub price: u64,
    pub nonce: [u8; 32],
}

pub fn decrypt_order(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<OrderDetails, EncryptionError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| EncryptionError::DecryptionFailed)?;
        
    let details: OrderDetails = serde_json::from_slice(&plaintext)
        .map_err(|_| EncryptionError::DecryptionFailed)?;
        
    Ok(details)
}
