# Jupiter DEX Substreams

[![Substreams](https://img.shields.io/badge/Substreams-v0.4.0-blue)](https://substreams.dev/packages/jupiter-dex-substreams/v0.4.0)
[![Solana](https://img.shields.io/badge/Network-Solana-purple)](https://solana.com)
[![Jupiter](https://img.shields.io/badge/DEX-Jupiter-orange)](https://jup.ag)
[![SQL Sink](https://img.shields.io/badge/SQL%20Sink-PostgreSQL%20%7C%20ClickHouse-green)](https://docs.substreams.dev)

High-performance Substreams for tracking Jupiter DEX aggregator events on Solana with **SQL sink support** and **75% data reduction** via foundational stores.

## Features

- **SQL Database Sink** - Stream swap data directly to PostgreSQL or ClickHouse
- **75% Data Reduction** - Uses foundational modules to filter vote transactions
- **Multi-Version Support** - Tracks Jupiter v1-v6, Limit Orders, and DCA
- **Comprehensive Analytics** - Volume tracking, unique traders, program stats
- **Production Ready** - Optimized Rust code with unit tests

## Quick Start

### Run with Substreams CLI

```bash
# Install Substreams CLI
curl -sSL https://substreams.dev/install.sh | bash

# Authenticate
substreams auth

# Run Jupiter analytics
substreams run https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.4.0/jupiter-dex-substreams-v0.4.0.spkg \
  map_jupiter_analytics \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
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
  jupiter-dex-substreams-v0.4.0.spkg

# 3. Run sink
substreams-sink-sql run \
  "psql://postgres:password@localhost:5432/jupiter?sslmode=disable" \
  jupiter-dex-substreams-v0.4.0.spkg
```

### Stream to ClickHouse

```bash
# 1. Start ClickHouse
docker run -d --name clickhouse \
  -p 8123:8123 -p 9000:9000 \
  clickhouse/clickhouse-server:latest

# 2. Setup schema (use clickhouse schema)
substreams-sink-sql setup \
  "clickhouse://default:@localhost:9000/default" \
  jupiter-dex-substreams-v0.4.0.spkg

# 3. Run sink
substreams-sink-sql run \
  "clickhouse://default:@localhost:9000/default" \
  jupiter-dex-substreams-v0.4.0.spkg
```

## Module Architecture

```
sf.solana.type.v1.Block
├── map_spl_initialized_account → AccountOwnerRecords
├── map_jupiter_trading_data → TradingDataList
│   ├── map_token_prices → TokenPriceList
│   ├── store_swap_volumes (bigint store)
│   └── store_unique_traders (string store)
├── map_jupiter_instructions → JupiterInstructions
│   └── map_jupiter_analytics → JupiterAnalytics
└── db_out → DatabaseChanges (SQL Sink)
```

## Available Modules

| Module | Output | Description |
|--------|--------|-------------|
| `map_jupiter_trading_data` | `TradingDataList` | Core swap data with parsed amounts |
| `map_jupiter_analytics` | `JupiterAnalytics` | Aggregated stats, top programs |
| `map_jupiter_instructions` | `JupiterInstructions` | Enriched instructions with ownership |
| `map_token_prices` | `TokenPriceList` | Token price calculations |
| `db_out` | `DatabaseChanges` | SQL sink output (PostgreSQL/ClickHouse) |

## Database Schema

### PostgreSQL Tables

| Table | Description |
|-------|-------------|
| `jupiter_swaps` | Individual swap events with amounts, mints, wallets |
| `daily_swap_stats` | Daily aggregated swap counts and volumes |
| `program_stats` | Per-program instruction counts and volumes |
| `global_metrics` | Protocol-wide totals |

### ClickHouse Tables

Same structure as PostgreSQL, optimized with:
- `MergeTree` engine with time-based partitioning
- `SummingMergeTree` for aggregations
- Materialized views for real-time analytics

## Jupiter Programs Tracked

| Program | Address | Type |
|---------|---------|------|
| Jupiter v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Swap Aggregator |
| Jupiter v4 | `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB` | Swap Aggregator |
| Jupiter v3 | `JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph` | Swap Aggregator |
| Jupiter v2 | `JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo` | Swap Aggregator |
| Limit Orders | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Limit Orders |
| DCA | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | Dollar Cost Averaging |

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

# Build
substreams build

# Run tests
cargo test

# Run with GUI
substreams gui substreams.yaml map_jupiter_analytics \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
```

### Project Structure

```
Jupiter-Dex-Substreams/
├── src/
│   ├── lib.rs                    # Module exports
│   ├── constants.rs              # Program IDs and configuration
│   ├── jupiter_trading_store.rs  # Core swap parsing (with tests)
│   ├── jupiter_instructions.rs   # Instruction enrichment
│   ├── jupiter_analytics.rs      # Analytics aggregation (with tests)
│   ├── token_price_store.rs      # Price calculations
│   ├── spl_account_store.rs      # SPL account tracking
│   ├── db_out.rs                 # SQL sink output (with tests)
│   └── pb/                       # Generated protobuf code
├── proto/
│   └── sf/jupiter/v1/types.proto # Data type definitions
├── schema.sql                    # PostgreSQL schema
├── schema.clickhouse.sql         # ClickHouse schema
├── substreams.yaml               # Manifest with sink config
└── Cargo.toml                    # Rust dependencies
```

## What's New in v0.4.0

### SQL Sink Support
- New `db_out` module for streaming to PostgreSQL/ClickHouse
- Pre-built schemas with indexes and materialized views
- Delta operations for efficient aggregations

### Performance Improvements
- Eliminated unnecessary string clones in hot paths
- Pre-allocated collections with estimated capacities
- Optimized instruction parsing with validation

### Code Quality
- Comprehensive unit tests for parsing and analytics
- Improved constants module with helper functions
- Better error handling with structured defaults

### Documentation
- Updated README with SQL sink instructions
- Added database schema documentation
- Improved inline code documentation

## Example Queries

### PostgreSQL

```sql
-- Top tokens by volume (last 24 hours)
SELECT input_mint, COUNT(*) as swaps, SUM(amount_in) as volume
FROM jupiter_swaps
WHERE block_time > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
GROUP BY input_mint
ORDER BY volume DESC
LIMIT 10;

-- Daily swap volume
SELECT date, swap_count, total_volume
FROM daily_swap_stats
ORDER BY date DESC
LIMIT 30;
```

### ClickHouse

```sql
-- Hourly volume with materialized view
SELECT hour, sum(swap_count), sum(volume_in)
FROM hourly_swap_volume
WHERE hour > now() - INTERVAL 24 HOUR
GROUP BY hour
ORDER BY hour;

-- Top trading pairs
SELECT input_mint, output_mint, swap_count, total_volume_in
FROM token_pair_volumes
ORDER BY swap_count DESC
LIMIT 20;
```

## Resources

- [Substreams Documentation](https://docs.substreams.dev/)
- [SQL Sink Guide](https://docs.substreams.dev/documentation/consume/sql)
- [Jupiter Developer Docs](https://docs.jup.ag/)
- [Solana Documentation](https://docs.solana.com/)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test`)
4. Commit changes (`git commit -m 'Add amazing feature'`)
5. Push to branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- **Package**: https://substreams.dev/packages/jupiter-dex-substreams/v0.4.0
- **Repository**: https://github.com/PaulieB14/Jupiter-Dex-Substreams
- **Jupiter**: https://jup.ag
- **Solana**: https://solana.com

---

**Built with Substreams for the Jupiter and Solana ecosystem**
