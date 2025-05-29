use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tracing::{error, info};

use crate::app_state::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(symbol): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| websocket_connection(socket, symbol, state))
}

async fn websocket_connection(socket: WebSocket, symbol: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut transaction_rx = state.tx.subscribe();

    info!("WebSocket connection established for symbol: {}", symbol);

    let symbol_clone = symbol.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        info!("Received message: {} for symbol: {}", text, symbol_clone);
                    }
                    Message::Close(_) => {
                        info!("WebSocket connection closed for symbol: {}", symbol_clone);
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
    });

    while let Ok(transaction) = transaction_rx.recv().await {
        if transaction.symbol == symbol {
            let json_msg = match serde_json::to_string(&transaction) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to serialize transaction: {}", e);
                    continue;
                }
            };

            if let Err(e) = sender.send(Message::Text(json_msg)).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection ended for symbol: {}", symbol);
} 