# Jupiter DEX Substreams

[![Substreams](https://img.shields.io/badge/Substreams-v0.5.0-blue)](https://substreams.dev)
[![Solana](https://img.shields.io/badge/Network-Solana-purple)](https://solana.com)
[![Jupiter](https://img.shields.io/badge/DEX-Jupiter-orange)](https://jup.ag)
[![SQL Sink](https://img.shields.io/badge/Sink-PostgreSQL%20%7C%20ClickHouse-green)](https://docs.substreams.dev)

High-performance Substreams for tracking Jupiter DEX aggregator on Solana with **OHLCV candles**, **SQL sink support**, **delta updates**, and comprehensive swap analytics.

## Features

| Feature | Description |
|---------|-------------|
| **OHLCV Candles** | Real-time candlestick data at 5min, 1hr, 4hr, and daily intervals |
| **SQL Database Sink** | Stream directly to PostgreSQL or ClickHouse |
| **Delta Updates** | Efficient aggregations using `set_if_null`, `max`, `min`, `add` operations |
| **Multi-Version Support** | Jupiter v2-v6, Limit Orders, and DCA programs |
| **Persistent Stores** | Track volumes, unique traders, and token stats across blocks |
| **Production Ready** | Optimized Rust with unit tests and comprehensive error handling |

## Quick Start

### Install & Authenticate

```bash
# Install Substreams CLI
curl -sSL https://substreams.dev/install.sh | bash

# Authenticate with StreamingFast
substreams auth
```

### Run Analytics

```bash
# Stream Jupiter analytics
substreams run jupiter-dex-substreams-v0.5.0.spkg \
  map_jupiter_analytics \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +100
```

### Stream to PostgreSQL

```bash
# 1. Start PostgreSQL
docker run -d --name postgres \
  -e POSTGRES_DB=jupiter \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:15

# 2. Setup schema
substreams-sink-sql setup \
  "psql://postgres:password@localhost:5432/jupiter?sslmode=disable" \
  jupiter-dex-substreams-v0.5.0.spkg

# 3. Run sink
substreams-sink-sql run \
  "psql://postgres:password@localhost:5432/jupiter?sslmode=disable" \
  jupiter-dex-substreams-v0.5.0.spkg
```

### Stream to ClickHouse

```bash
# 1. Start ClickHouse
docker run -d --name clickhouse \
  -p 8123:8123 -p 9000:9000 \
  clickhouse/clickhouse-server:latest

# 2. Setup and run with ClickHouse engine
substreams-sink-sql setup \
  "clickhouse://default:@localhost:9000/default" \
  jupiter-dex-substreams-v0.5.0.spkg \
  --engine=clickhouse

substreams-sink-sql run \
  "clickhouse://default:@localhost:9000/default" \
  jupiter-dex-substreams-v0.5.0.spkg \
  --engine=clickhouse
```

## Architecture

```
sf.solana.type.v1.Block
│
├─► map_spl_initialized_account ──► AccountOwnerRecords
│
├─► map_jupiter_trading_data ──► TradingDataList
│   │
│   ├─► map_token_prices ──► TokenPriceList
│   │
│   ├─► store_swap_volumes (bigint, add)
│   │   └─► pair:{in}:{out}, token:volume_in:{mint}, daily:{date}:volume
│   │
│   ├─► store_unique_traders (string, set_if_not_exists)
│   │   └─► trader:{wallet}, daily:{date}:trader:{wallet}
│   │
│   └─► store_token_stats (bigint, add)
│       └─► token:{mint}:trade_count
│
├─► map_jupiter_instructions ──► JupiterInstructions
│   │
│   └─► map_jupiter_analytics ──► JupiterAnalytics
│
└─► db_out ──► DatabaseChanges (SQL Sink)
    │
    ├─► jupiter_swaps (individual trades)
    ├─► candles (OHLCV at 5m/1h/4h/1d)
    ├─► token_pairs (pair statistics)
    ├─► token_stats (per-token metrics)
    ├─► trader_stats (wallet activity)
    ├─► daily_stats / hourly_stats
    ├─► program_stats (per-version)
    └─► protocol_metrics (global totals)
```

## Database Schema

### Core Tables

| Table | Description | Delta Operations |
|-------|-------------|------------------|
| `jupiter_swaps` | Individual swap events | `create_row` |
| `candles` | OHLCV candlestick data | `set_if_null(open)`, `set(close)`, `max(high)`, `min(low)`, `add(volume)` |
| `token_pairs` | Trading pair statistics | `add(swap_count, volume)`, `set(last_swap)` |
| `token_stats` | Per-token metrics | `add(swaps, volume)`, `set(last_seen)` |
| `trader_stats` | Wallet activity | `add(swaps, volume)`, `set(last_swap)` |
| `daily_stats` | Daily aggregations | `add(swap_count, volume)` |
| `hourly_stats` | Hourly aggregations | `add(swap_count, volume)` |
| `program_stats` | Per-program stats | `add(count, volume)` |
| `protocol_metrics` | Global protocol metrics | `add(swaps, volume)`, `max(unique_*)` |

### Candle Intervals

| Interval | Seconds | Use Case |
|----------|---------|----------|
| 5 minutes | 300 | High-frequency trading, scalping |
| 1 hour | 3600 | Intraday analysis |
| 4 hours | 14400 | Swing trading |
| 1 day | 86400 | Long-term trends |

### Views (PostgreSQL)

```sql
-- Get latest prices for all pairs
SELECT * FROM latest_prices;

-- Top tokens by volume
SELECT * FROM top_tokens_24h;

-- Top trading pairs
SELECT * FROM top_pairs;

-- Top traders by volume
SELECT * FROM top_traders;

-- Daily volume trend (30 days)
SELECT * FROM daily_volume_trend;

-- Hourly volume (24 hours)
SELECT * FROM hourly_volume_24h;

-- Program distribution
SELECT * FROM program_distribution;

-- 5-minute candles (24h)
SELECT * FROM candles_5m_24h WHERE pair_id = 'SOL:USDC';

-- Hourly candles (7 days)
SELECT * FROM candles_1h_7d WHERE pair_id = 'SOL:USDC';

-- Daily candles (30 days)
SELECT * FROM candles_1d_30d WHERE pair_id = 'SOL:USDC';
```

### Views (ClickHouse)

```sql
-- Finalized hourly candles
SELECT * FROM v_candles_1h WHERE pair_id = 'SOL:USDC' ORDER BY candle_time DESC;

-- Finalized daily candles
SELECT * FROM v_candles_1d WHERE pair_id = 'SOL:USDC' ORDER BY candle_time DESC;

-- Top tokens
SELECT * FROM v_top_tokens;

-- User activity
SELECT * FROM v_user_activity LIMIT 100;

-- Daily unique users
SELECT * FROM v_daily_unique_users;
```

## Jupiter Programs Tracked

| Program | Address | Type |
|---------|---------|------|
| Jupiter v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Swap Aggregator (Latest) |
| Jupiter v4 | `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB` | Swap Aggregator |
| Jupiter v3 | `JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph` | Swap Aggregator |
| Jupiter v2 | `JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo` | Swap Aggregator |
| Limit Orders | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Limit Orders |
| DCA | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | Dollar Cost Averaging |

## Example Queries

### Get Candles for a Trading Pair

```sql
-- PostgreSQL: Use the helper function
SELECT * FROM get_candles('SOL_MINT:USDC_MINT', 3600, 24);

-- Or query directly
SELECT
    timestamp,
    open,
    high,
    low,
    close,
    volume_in,
    trade_count
FROM candles
WHERE pair_id = 'So11111111111111111111111111111111111111112:EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'
  AND interval_seconds = 3600
ORDER BY timestamp DESC
LIMIT 24;
```

### Top Tokens by Volume

```sql
-- PostgreSQL
SELECT
    mint_address,
    total_swaps_as_input + total_swaps_as_output AS total_swaps,
    total_volume_as_input + total_volume_as_output AS total_volume
FROM token_stats
ORDER BY total_volume DESC
LIMIT 20;

-- ClickHouse
SELECT * FROM v_top_tokens;
```

### Whale Activity (Large Trades)

```sql
SELECT
    tx_hash,
    user_wallet,
    input_mint,
    output_mint,
    amount_in,
    amount_out,
    TO_TIMESTAMP(block_time) as swap_time
FROM jupiter_swaps
WHERE amount_in > 1000000000000  -- > 1M tokens (adjust for decimals)
ORDER BY block_time DESC
LIMIT 100;
```

### Trading Pair Analysis

```sql
SELECT
    input_mint,
    output_mint,
    swap_count,
    total_volume_in,
    total_volume_out,
    TO_TIMESTAMP(last_swap_time) as last_trade
FROM token_pairs
ORDER BY swap_count DESC
LIMIT 50;
```

## Development

### Prerequisites

- Rust 1.70+
- Substreams CLI 1.7.0+
- `buf` CLI (for protobuf generation)

### Build

```bash
# Clone repository
git clone https://github.com/PaulieB14/Jupiter-Dex-Substreams.git
cd Jupiter-Dex-Substreams

# Build WASM
substreams build

# Run tests
cargo test

# Run with GUI
substreams gui substreams.yaml map_jupiter_analytics \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +100
```

### Project Structure

```
Jupiter-Dex-Substreams/
├── src/
│   ├── lib.rs                    # Module exports
│   ├── constants.rs              # Program IDs
│   ├── jupiter_trading_store.rs  # Core swap parsing
│   ├── jupiter_instructions.rs   # Instruction enrichment
│   ├── jupiter_analytics.rs      # Analytics aggregation
│   ├── token_price_store.rs      # Price tracking
│   ├── spl_account_store.rs      # Account ownership
│   ├── stores.rs                 # Persistent stores
│   ├── db_out.rs                 # SQL sink with candles
│   └── pb/                       # Generated protobuf
├── proto/
│   └── sf/jupiter/v1/types.proto # Data types
├── schema.sql                    # PostgreSQL schema
├── schema.clickhouse.sql         # ClickHouse schema
├── substreams.yaml               # Manifest
└── Cargo.toml                    # Dependencies
```

## What's New in v0.5.0

### OHLCV Candles
- Real-time candlestick data for all trading pairs
- Multiple intervals: 5min, 1hr, 4hr, 1day
- Delta updates: `set_if_null(open)`, `set(close)`, `max(high)`, `min(low)`, `add(volume)`

### Enhanced Tables
- `candles` - OHLCV data with composite primary key
- `token_pairs` - Trading pair statistics
- `token_stats` - Per-token metrics
- `trader_stats` - Wallet activity tracking
- `hourly_stats` - Hourly aggregations

### Persistent Stores
- `store_swap_volumes` - Cumulative volumes by pair, token, date
- `store_unique_traders` - First-seen tracking for wallets
- `store_token_stats` - Trade counts per token

### ClickHouse Optimizations
- Materialized views for real-time candle aggregation
- `AggregatingMergeTree` for efficient state management
- Pre-computed views for common queries

## Resources

- [Substreams Documentation](https://docs.substreams.dev/)
- [SQL Sink Guide](https://docs.substreams.dev/documentation/consume/sql)
- [Delta Updates Demo](https://github.com/streamingfast/substreams-eth-uni-v4-demo-candles)
- [Jupiter Developer Docs](https://docs.jup.ag/)
- [Solana Documentation](https://docs.solana.com/)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/awesome`)
3. Run tests (`cargo test`)
4. Commit changes (`git commit -m 'Add awesome feature'`)
5. Push to branch (`git push origin feature/awesome`)
6. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Built with Substreams for the Jupiter and Solana ecosystem**
