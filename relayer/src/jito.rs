use reqwest::Client;
use serde_json::json;
use solana_sdk::transaction::Transaction;
use thiserror::Error;
use base64::{Engine as _, engine::general_purpose::STANDARD as base64_standard};

#[derive(Error, Debug)]
pub enum JitoError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Jito API error: {0}")]
    ApiError(String),
}

pub struct JitoClient {
    client: Client,
    endpoint: String,
}

impl JitoClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn submit_bundle(&self, txs: Vec<Transaction>) -> Result<String, JitoError> {
        let serialized_txs: Vec<String> = txs.iter()
            .map(|tx| {
                let serialized = bincode::serialize(tx).unwrap();
                base64_standard.encode(&serialized)
            })
            .collect();

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [
                serialized_txs
            ]
        });

        let res = self.client.post(&self.endpoint)
            .json(&payload)
            .send()
            .await?;

        let res_json: serde_json::Value = res.json().await?;
        
        if let Some(err) = res_json.get("error") {
            return Err(JitoError::ApiError(err.to_string()));
        }

        let bundle_id = res_json["result"].as_str()
            .unwrap_or_default()
            .to_string();

        Ok(bundle_id)
    }
}
