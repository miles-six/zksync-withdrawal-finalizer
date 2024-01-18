use axum::extract::State;
use axum::{http::StatusCode, routing::get, Json, Router};
use ethers::abi::Address;
use ethers::types::{H256, U256};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use storage::UserWithdrawal;

#[derive(Deserialize, Serialize, Clone)]
struct WithdrawalRequest {
    pub from: Address,
    pub limit: u64,
}
#[derive(Deserialize, Serialize, Clone)]
struct WithdrawalResponse {
    pub tx_hash: H256,
    pub token: Address,
    pub amount: U256,
    pub status: String,
}

impl From<UserWithdrawal> for WithdrawalResponse {
    fn from(withdrawal: UserWithdrawal) -> Self {
        Self {
            tx_hash: withdrawal.tx_hash,
            token: withdrawal.token,
            amount: withdrawal.amount,
            status: format!("{:?}", withdrawal.status),
        }
    }
}

pub async fn run_server(pool: PgPool) {
    let app = Router::new()
        .route("/withdrawals", get(get_withdrawals))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_withdrawals(
    State(pool): State<PgPool>,
    Json(payload): Json<WithdrawalRequest>,
) -> Result<Json<Vec<WithdrawalResponse>>, StatusCode> {
    let result: Vec<_> = storage::withdrawals_for_address(&pool, payload.from, payload.limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(WithdrawalResponse::from)
        .collect();
    Ok(Json(result))
}