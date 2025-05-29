use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::models::kline::KlineResponse;

#[derive(Deserialize)]
pub struct KlineQuery {
    pub interval: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_klines(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<KlineQuery>,
) -> Result<Json<KlineResponse>, StatusCode> {
    let interval = params.interval.unwrap_or_else(|| "1m".to_string());
    let limit = params.limit;

    // validate interval
    let valid_intervals = ["1s", "1m", "5m", "15m", "1h"];
    if !valid_intervals.contains(&interval.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let klines = state
        .kline_service
        .get_klines(&symbol, &interval, limit)
        .await;

    Ok(Json(KlineResponse {
        symbol,
        interval,
        data: klines,
    }))
} 