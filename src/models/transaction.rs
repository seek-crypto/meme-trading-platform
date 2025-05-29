use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub symbol: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl Transaction {
    pub fn new(symbol: String, price: f64, amount: f64, side: Side) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            symbol,
            price,
            amount,
            side,
            timestamp: Utc::now(),
        }
    }
} 