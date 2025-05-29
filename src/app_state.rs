use std::sync::Arc;
use tokio::sync::broadcast;

use crate::models::transaction::Transaction;
use crate::services::{kline_service::KlineService, stream_service::StreamService};

pub struct AppState {
    pub kline_service: Arc<KlineService>,
    pub stream_service: Arc<StreamService>,
    pub tx: broadcast::Sender<Transaction>,
} 