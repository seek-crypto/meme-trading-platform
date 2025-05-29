use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub symbol: String,
    pub interval: String,
}

impl Kline {
    pub fn new(timestamp: DateTime<Utc>, symbol: String, interval: String) -> Self {
        Self {
            timestamp,
            open: 0.0,
            high: 0.0,
            low: f64::MAX, // set to max so first price becomes low
            close: 0.0,
            volume: 0.0,
            symbol,
            interval,
        }
    }

    pub fn update_with_price(&mut self, price: f64, volume: f64) {
        if self.open == 0.0 {
            self.open = price;
        }
        
        if price > self.high {
            self.high = price;
        }
        
        if price < self.low {
            self.low = price;
        }
        
        self.close = price;
        self.volume += volume;
    }
}

#[derive(Debug, Serialize)]
pub struct KlineResponse {
    pub symbol: String,
    pub interval: String,
    pub data: Vec<Kline>,
} 