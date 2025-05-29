use crate::models::{kline::Kline, transaction::Transaction};
use crate::utils::time_utils;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct KlineService {
    // symbol -> interval -> klines
    klines: Arc<DashMap<String, DashMap<String, VecDeque<Kline>>>>,
    // symbol -> interval -> current open kline
    current_klines: Arc<DashMap<String, DashMap<String, Kline>>>,
}

impl KlineService {
    pub fn new() -> Self {
        Self {
            klines: Arc::new(DashMap::new()),
            current_klines: Arc::new(DashMap::new()),
        }
    }

    pub async fn update_with_transaction(&self, transaction: &Transaction) {
        let intervals = vec!["1s", "1m", "5m", "15m", "1h"];
        
        for interval in intervals {
            self.update_kline(&transaction.symbol, interval, transaction).await;
        }
    }

    async fn update_kline(&self, symbol: &str, interval: &str, transaction: &Transaction) {
        let timestamp = time_utils::round_to_interval(transaction.timestamp, interval);
        
        // Get or create current kline
        let current_klines = self.current_klines
            .entry(symbol.to_string())
            .or_insert_with(DashMap::new);
        
        let mut current_kline = current_klines
            .entry(interval.to_string())
            .or_insert_with(|| Kline::new(timestamp, symbol.to_string(), interval.to_string()));

        // Check if we need to close current kline and start a new one
        if current_kline.timestamp != timestamp {
            // Close the previous kline
            let closed_kline = current_kline.clone();
            self.store_closed_kline(symbol, interval, closed_kline).await;
            
            // Start new kline
            *current_kline = Kline::new(timestamp, symbol.to_string(), interval.to_string());
        }

        // Update current kline with transaction
        current_kline.update_with_price(transaction.price, transaction.amount);
    }

    async fn store_closed_kline(&self, symbol: &str, interval: &str, kline: Kline) {
        let klines = self.klines
            .entry(symbol.to_string())
            .or_insert_with(DashMap::new);
        
        let mut interval_klines = klines
            .entry(interval.to_string())
            .or_insert_with(VecDeque::new);

        interval_klines.push_back(kline);
        
        // TODO: make this configurable
        if interval_klines.len() > 1000 {
            interval_klines.pop_front();
        }
    }

    pub async fn get_klines(&self, symbol: &str, interval: &str, limit: Option<usize>) -> Vec<Kline> {
        let limit = limit.unwrap_or(100).min(1000);
        let mut result = Vec::new();

        // Get historical klines
        if let Some(klines) = self.klines.get(symbol) {
            if let Some(interval_klines) = klines.get(interval) {
                let start_index = if interval_klines.len() > limit {
                    interval_klines.len() - limit
                } else {
                    0
                };
                
                result.extend(
                    interval_klines
                        .range(start_index..)
                        .cloned()
                );
            }
        }

        // Add current open kline if exists
        if let Some(current_klines) = self.current_klines.get(symbol) {
            if let Some(current_kline) = current_klines.get(interval) {
                result.push(current_kline.clone());
            }
        }

        result
    }
} 