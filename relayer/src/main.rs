mod api;
mod encryption;
mod jito;
mod matcher;
mod proof;

use api::{create_router, AppState};
use matcher::MatchingEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    println!("Starting Bastion Relayer Engine...");

    // F8 FIX: Load relayer key from environment variable
    let relayer_key_hex = std::env::var("BASTION_RELAYER_KEY")
        .expect("BASTION_RELAYER_KEY environment variable must be set (64 hex chars)");

    let relayer_key_bytes = hex::decode(&relayer_key_hex)
        .expect("BASTION_RELAYER_KEY must be valid hex");

    let mut relayer_key = [0u8; 32];
    if relayer_key_bytes.len() != 32 {
        panic!("BASTION_RELAYER_KEY must be exactly 32 bytes (64 hex chars)");
    }
    relayer_key.copy_from_slice(&relayer_key_bytes);

    // F14: Load API key from environment
    let api_key = std::env::var("BASTION_API_KEY")
        .unwrap_or_else(|_| {
            eprintln!("WARNING: BASTION_API_KEY not set. Using default (insecure for production).");
            "bastion-dev-key-change-me".to_string()
        });

    let state = Arc::new(AppState {
        matcher: Mutex::new(MatchingEngine::new()),
        relayer_key,
        api_key,
    });

    let app = create_router(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
