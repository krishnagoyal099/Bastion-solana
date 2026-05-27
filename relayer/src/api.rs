use axum::{
    routing::{post, get},
    Router, Json, extract::State,
    http::{Request, StatusCode, HeaderMap},
    middleware::{self, Next},
    response::Response,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use solana_sdk::pubkey::Pubkey;
use crate::matcher::MatchingEngine;
use crate::encryption::{decrypt_order, OrderDetails};
use crate::proof::verify_proof_hash;

pub struct AppState {
    pub matcher: Mutex<MatchingEngine>,
    pub relayer_key: [u8; 32],
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct SubmitOrderRequest {
    pub commitment: [u8; 32],
    pub beneficiary: String,
    pub proof_bytes: Vec<u8>,
    pub proof_hash: [u8; 32],
    pub encrypted_details: Vec<u8>,
    pub aes_nonce: [u8; 12],
}

#[derive(Serialize)]
pub struct SubmitOrderResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct OrderBookResponse {
    pub buy_count: usize,
    pub sell_count: usize,
}

/// F14 FIX: API key authentication middleware
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(key) if key == state.api_key => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn submit_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitOrderRequest>,
) -> Json<SubmitOrderResponse> {
    // Validate proof hash
    if let Err(_) = verify_proof_hash(&payload.proof_bytes, &payload.proof_hash) {
        return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Invalid ZK proof hash".into(),
        });
    }

    // Reject zero commitments
    if payload.commitment == [0u8; 32] {
        return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Zero commitment not allowed".into(),
        });
    }

    // Decrypt order details
    let details = match decrypt_order(&state.relayer_key, &payload.encrypted_details, &payload.aes_nonce) {
        Ok(d) => d,
        Err(_) => return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Failed to decrypt order details".into(),
        }),
    };

    // Validate order details
    if details.side > 1 {
        return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Invalid order side (must be 0=buy or 1=sell)".into(),
        });
    }
    if details.amount == 0 || details.price == 0 {
        return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Amount and price must be > 0".into(),
        });
    }

    use std::str::FromStr;
    let beneficiary = match Pubkey::from_str(&payload.beneficiary) {
        Ok(p) => p,
        Err(_) => return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Invalid beneficiary pubkey".into(),
        }),
    };

    let mut matcher = state.matcher.lock().await;

    // F14: Bound order book size to prevent OOM
    const MAX_ORDER_BOOK_SIZE: usize = 10_000;
    if matcher.buys.len() + matcher.sells.len() >= MAX_ORDER_BOOK_SIZE {
        return Json(SubmitOrderResponse {
            status: "error".into(),
            message: "Order book full, try again later".into(),
        });
    }

    matcher.add_order(payload.commitment, beneficiary, details);

    if let Some((buy, sell, price)) = matcher.match_orders() {
        println!("Match found! Buy: {:?}, Sell: {:?} @ Price {}", buy.commitment, sell.commitment, price);
        // In production: construct Jito bundle and submit to chain
    }

    Json(SubmitOrderResponse {
        status: "success".into(),
        message: "Order queued".into(),
    })
}

pub async fn get_order_book(
    State(state): State<Arc<AppState>>,
) -> Json<OrderBookResponse> {
    let matcher = state.matcher.lock().await;
    Json(OrderBookResponse {
        buy_count: matcher.buys.len(),
        sell_count: matcher.sells.len(),
    })
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/order", post(submit_order))
        .route("/orderbook", get(get_order_book))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}
