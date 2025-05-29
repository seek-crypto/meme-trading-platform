use crate::models::transaction::{Side, Transaction};
use crate::services::kline_service::KlineService;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use tracing::info;

pub struct DataGenerator {
    symbols: Vec<String>,
    base_prices: std::collections::HashMap<String, f64>,
}

impl DataGenerator {
    pub fn new() -> Self {
        let symbols = vec![
            "PEPE".to_string(),
            "DOGE".to_string(),
            "SHIB".to_string(),
            "FLOKI".to_string(),
        ];

        let mut base_prices = std::collections::HashMap::new();
        // rough estimates of real prices
        base_prices.insert("PEPE".to_string(), 0.000001234);
        base_prices.insert("DOGE".to_string(), 0.0823);
        base_prices.insert("SHIB".to_string(), 0.000008456);
        base_prices.insert("FLOKI".to_string(), 0.00012345);

        Self {
            symbols,
            base_prices,
        }
    }

    pub async fn start_generation(
        &self,
        tx: broadcast::Sender<Transaction>,
        kline_service: Arc<KlineService>,
    ) {
        info!("Starting data generation...");
        
        let mut interval = time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            let transaction = {
                let mut rng = rand::thread_rng();
                let symbol = self.symbols[rng.gen_range(0..self.symbols.len())].clone();
                let base_price = *self.base_prices.get(&symbol).unwrap();
                
                // add some randomness ±5%
                let price_variation = rng.gen_range(-0.05..0.05);
                let price = base_price * (1.0 + price_variation);
                
                let amount = rng.gen_range(100.0..10000.0);
                let side = if rng.gen_bool(0.5) { Side::Buy } else { Side::Sell };

                Transaction::new(symbol, price, amount, side)
            };

            kline_service.update_with_transaction(&transaction).await;

            if let Err(e) = tx.send(transaction.clone()) {
                tracing::warn!("Failed to broadcast transaction: {}", e);
            }
        }
    }
} 