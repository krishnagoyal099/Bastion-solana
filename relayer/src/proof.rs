use sha2::{Sha256, Digest};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofError {
    #[error("Invalid proof hash")]
    InvalidHash,
}

pub fn verify_proof_hash(proof_bytes: &[u8], expected_hash: &[u8; 32]) -> Result<(), ProofError> {
    let mut hasher = Sha256::new();
    hasher.update(proof_bytes);
    let result = hasher.finalize();
    
    if result.as_slice() != expected_hash {
        return Err(ProofError::InvalidHash);
    }
    
    Ok(())
}
