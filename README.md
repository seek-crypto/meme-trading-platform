# Meme Trading Platform 

Just a simple real-time data service for meme token trading. Built with Rust for the performance and memory safety - perfect for high-frequency trading data.

## What it does

- Real-time candlestick data for meme coins (1s, 1m, 5m, 15m, 1h intervals)
- Live transaction streaming via WebSocket
- Supports PEPE, DOGE, SHIB, FLOKI (the usual suspects)
- Updates current candles in real-time
- Pretty fast thanks to Rust + async

## Getting started

Need Rust 1.75+ installed. That's about it.

```bash
git clone <this-repo>
cd meme-trading-platform
cargo run
```

Should start on localhost:3000. If port 3000 is busy, kill whatever's using it first.

### Docker route (if you prefer)
```bash
docker build -t meme-trading .
docker run -p 3000:3000 meme-trading
```

## API stuff

### Get candlestick data
```
GET /api/klines/{symbol}?interval={interval}&limit={limit}
```

Example:
```bash
curl "http://localhost:3000/api/klines/PEPE?interval=1m&limit=10"
```

Returns JSON with OHLCV data. Pretty standard stuff.

### WebSocket for live data
```
ws://localhost:3000/ws/{symbol}
```

Streams live transactions as they happen. Each message looks like:
```json
{
  "id": "some-uuid",
  "symbol": "PEPE", 
  "price": 0.000001234,
  "amount": 1000000.0,
  "side": "Buy",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Health check
```
GET /health
```
Returns "OK" if everything's working.

## Testing it out

Start the server, then:

```bash
# Check if it's alive
curl http://localhost:3000/health

# Get some PEPE data
curl "http://localhost:3000/api/klines/PEPE?interval=5m&limit=5"

# Test WebSocket (if you have websocat)
websocat ws://localhost:3000/ws/DOGE
```

## How it works

```
Data Generator → KlineService → API/WebSocket
     ↓              ↓              ↓
  Mock trades   Aggregates    Serves data
  every 100ms   into bars     to clients
```

Nothing fancy. Generator creates fake trades, KlineService turns them into candlestick bars, and the API serves it up.

## Integration notes

If you want to plug this into a bigger trading platform, here's how I'd do it:

### Frontend integration
Pretty straightforward. Any chart library can consume the REST API:

```javascript
// Basic usage - just fetch and display
const response = await fetch('/api/klines/PEPE?interval=1m&limit=100');
const klines = await response.json();

// WebSocket for live updates
const ws = new WebSocket('ws://localhost:3000/ws/PEPE');
ws.onmessage = (event) => {
  const trade = JSON.parse(event.data);
  // update your charts, trading feed, whatever
  updatePriceDisplay(trade.symbol, trade.price);
  addToTransactionFeed(trade);
};

// For trading charts, you'd probably want something like:
const chartData = klines.data.map(k => ({
  timestamp: k.timestamp,
  open: k.open,
  high: k.high,
  low: k.low,
  close: k.close,
  volume: k.volume
}));
```

### Backend integration
The service is designed to be pretty modular. You could:

**1. Replace mock data with real feeds:**
```rust
// Instead of DataGenerator, plug in real exchange APIs
impl RealDataIngester {
    async fn connect_to_exchange(&self) {
        // Connect to Binance, Coinbase, whatever
        // Parse their WebSocket feeds
        // Feed into our KlineService
    }
}
```

**2. Add a trade ingestion endpoint:**
```rust
// POST /internal/trades - for other services to send trade data
async fn ingest_trade(Json(trade): Json<Trade>) -> StatusCode {
    kline_service.update_with_transaction(&trade).await;
    broadcast_tx.send(trade).ok();
    StatusCode::OK
}
```

**3. Authentication layer:**
Right now there's zero auth. In production you'd probably want:
- API keys for REST endpoints
- JWT tokens for WebSocket connections
- Rate limiting (maybe 100 req/min per key?)

### Database integration
Currently everything lives in memory, which is fast but not persistent. For production:

**Historical data storage:**
```rust
// PostgreSQL for long-term storage
CREATE TABLE klines (
    symbol VARCHAR(10),
    interval VARCHAR(5),
    timestamp TIMESTAMPTZ,
    open DECIMAL,
    high DECIMAL,
    low DECIMAL,
    close DECIMAL,
    volume DECIMAL
);

// Index on (symbol, interval, timestamp) for fast queries
```

**Real-time caching:**
```rust
// Redis for current/hot data
// Key pattern: "kline:{symbol}:{interval}:current"
// TTL based on interval (1m bars expire after 1 minute, etc.)
```

**Message queues for scale:**
```rust
// Kafka topics for distributing trade data
// trades.raw -> all incoming trades
// trades.{symbol} -> symbol-specific trades
// klines.{interval} -> completed kline bars
```

### Scaling strategy

**Horizontal scaling:**
- Run multiple instances behind nginx/HAProxy
- Share state via Redis/PostgreSQL
- Use sticky sessions for WebSocket connections (or don't, and handle reconnects)

**Data partitioning:**
```rust
// Could split by symbol
// Service A: handles PEPE, DOGE
// Service B: handles SHIB, FLOKI
// Load balancer routes by symbol
```

**Performance tricks I'd add:**
- Connection pooling for DB
- Batch writes (collect 100 trades, write once)
- Separate read/write replicas
- CDN for historical data API

### Integration with larger platform

The way I see it fitting into a full trading platform:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Web UI    │    │ Mobile App  │    │ Admin Panel │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
        ┌─────────────────────────────────────┐
        │        API Gateway (nginx)          │
        └─────────────────┬───────────────────┘
                          │
    ┌─────────────────────┼─────────────────────┐
    │                     │                     │
┌───▼──────┐  ┌──────▼────┐  ┌─────▼─────┐  ┌─────────┐
│ Trading  │  │   Data    │  │   User    │  │ Payment │
│ Engine   │  │ Service   │  │   Mgmt    │  │ Service │
│          │  │  (this)   │  │           │  │         │
└─────┬────┘  └─────┬─────┘  └───────────┘  └─────────┘
      │             │
      └─────────────┘
      │
┌─────▼─────┐
│   Kafka   │  ← All services pub/sub here
│  Cluster  │
└───────────┘
```

**Service communication:**
- Trading Engine sends completed trades → Data Service
- Data Service broadcasts price updates → All UIs
- Admin Panel queries Data Service for charts/analytics

**Failure handling:**
- If Data Service goes down, Trading Engine keeps working
- WebSocket clients auto-reconnect
- Historical data still available from DB

This is definitely MVP-level right now, but the architecture should scale pretty well. The main bottleneck would probably be WebSocket connections - might need to add a dedicated WebSocket service layer eventually.

## Project structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Module exports
├── app_state.rs         # Shared app state
├── models/              # Data structures
│   ├── kline.rs         # Candlestick data
│   └── transaction.rs   # Trade data
├── services/            # Business logic
│   ├── data_generator.rs # Mock data
│   ├── kline_service.rs  # Bar aggregation
│   └── stream_service.rs # WebSocket management
├── handlers/            # HTTP/WS handlers
│   ├── kline_handler.rs  # REST API
│   └── websocket_handler.rs # WebSocket
└── utils/
    └── time_utils.rs    # Time calculations
```

## Running tests

```bash
cargo test
```

Integration tests need the server running:
```bash
cargo run &
cargo test --test integration_tests
kill %1  # stop background server
```

## Performance notes

- Uses DashMap for concurrent access
- Broadcast channels for WebSocket distribution  
- Limits history to 1000 bars per interval (configurable)
- Should handle a few thousand concurrent WebSocket connections

## TODO / Ideas

- [ ] Real data integration (CoinGecko API maybe?)
- [ ] Database persistence
- [ ] Rate limiting
- [ ] Admin dashboard
- [ ] More technical indicators
- [ ] Order book simulation
- [ ] Better error handling
- [ ] Metrics/monitoring

## Known issues

- All data is fake/simulated
- No authentication 
- In-memory only (data lost on restart)
- Single instance (no clustering)
- Minimal error handling


## Environment

Set `RUST_LOG=debug` for more verbose logging.
Port defaults to 3000, override with `PORT` env var.

