# Jupiter DEX Events Substream

[![Substreams](https://img.shields.io/badge/Substreams-v0.3.3-blue)](https://substreams.dev/packages/jupiter-dex-substreams/v0.3.3)
[![Solana](https://img.shields.io/badge/Network-Solana-purple)](https://solana.com)
[![Jupiter](https://img.shields.io/badge/DEX-Jupiter-orange)](https://jup.ag)
[![Performance](https://img.shields.io/badge/Performance-75%25%20Faster-green)](https://docs.substreams.dev)

A high-performance Substreams package for tracking Jupiter DEX aggregator events on Solana blockchain with **75% data reduction** via foundational stores.

## 🌟 Overview

This substream provides complete visibility into Jupiter's DEX aggregation ecosystem, tracking:

- **🔄 Jupiter Swap Events** (v1-v6): Token swap aggregations across multiple DEXs
- **📋 Jupiter Limit Orders**: Advanced order management functionality  
- **💰 Jupiter DCA**: Dollar Cost Averaging automation
- **🧠 Aggregation Events**: Cross-DEX routing decisions and arbitrage opportunities

## 🚀 Key Features

- **⚡ 75% Data Reduction**: Uses solana-common foundational modules to exclude voting transactions
- **🎯 Block Filtering**: Efficient filtering skips irrelevant blocks before processing
- **📊 Performance Stores**: Track swap volumes and unique traders for analytics
- **🔄 Multi-Version Support**: Tracks Jupiter v1-v6 simultaneously
- **🌐 Cross-DEX Analysis**: Understands routing across Raydium, Orca, and other DEXs
- **📈 Real-time Events**: Live tracking of Jupiter's aggregation decisions
- **💾 Shared Caching**: Leverages foundational stores for reduced costs

## 📈 Performance Characteristics

- **Data Reduction**: 75% via vote transaction filtering
- **Query Performance**: 3-5x faster through foundational store caching
- **Block Range**: Optimized for blocks 31,310,775+
- **Parallel Processing**: Enabled for all map modules
- **Infrastructure Cost**: Significantly reduced via shared caching

## 📦 Installation

```bash
# Install Substreams CLI
curl -sSL https://substreams.dev/install.sh | bash

# Run Jupiter events (v0.3.3)
substreams run https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.3.3/jupiter-dex-substreams-v0.3.3.spkg \
  map_jupiter_instructions \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1
```

## 🏗️ Module Architecture

```
jupiter_filtered_transactions (base module with foundational stores)
├── map_spl_initialized_account
├── map_jupiter_trading_data
│   ├── map_token_prices
│   ├── store_swap_volumes
│   └── store_unique_traders
├── map_jupiter_instructions
└── map_jupiter_analytics
```

## 🏗️ Development

### Prerequisites

- Rust 1.70+
- Substreams CLI 1.7.0+
- Solana CLI (optional)

### Build

```bash
# Clone the repository
git clone https://github.com/PaulieB14/Jupiter-Dex-Substreams.git
cd Jupiter-Dex-Substreams

# Build the WASM + package
substreams build

# (Optional) compile-check without packaging
cargo check --target wasm32-unknown-unknown

# Run with GUI for visualization
substreams gui substreams.yaml map_jupiter_instructions \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
```

## 📊 Event Types

### Swap Events
- Route discovery and execution
- Multi-hop swap tracking  
- Slippage and price impact
- Cross-DEX routing decisions

### Limit Order Events
- Order placement and management
- Order execution and fulfillment
- Order cancellation events

### DCA Events
- Scheduled purchase events
- DCA execution status
- Position management

### Aggregation Events
- Cross-DEX arbitrage opportunities
- Liquidity source selection
- Route optimization decisions

## 🔧 Jupiter Program IDs

| Program | Address | Version |
|---------|---------|---------|
| Jupiter Swap v6 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Latest |
| Jupiter Swap v4/v3 | `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB` | v4/v3 |
| Jupiter Swap v2 | `JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph` | v2 |
| Jupiter Swap v1 | `JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo` | v1 |
| Jupiter Limit Orders | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Orders |
| Jupiter DCA | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | DCA |

## 📈 Usage Examples

### Basic Usage
```bash
# Run with GUI
substreams gui substreams.yaml map_jupiter_analytics

# Run specific block range
substreams run substreams.yaml map_jupiter_instructions \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
```

### Advanced Usage
```bash
# Access swap volume store
substreams run substreams.yaml store_swap_volumes \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +100

# Run with parallelization
substreams run substreams.yaml map_jupiter_analytics \
  -e mainnet.sol.streamingfast.io:443 \
  -H "X-Substreams-Parallel-Workers: 20" \
  -s 325766951 -t +1000
```

## 🏗️ Architecture

This substream is built using:

- **Rust** for the core logic
- **Protocol Buffers** for data serialization  
- **Substreams Solana** for blockchain data access
- **Solana Common v0.3.0** for foundational modules
- **Block Filtering** for efficient data processing

## 🆕 What's New in v0.3.3

### Swap Amount Parsing
- ✅ Parse Jupiter v6 instruction data for swap amounts
- ✅ Extract `amount_in`, `amount_out`, `input_mint`, `output_mint`
- ✅ Support for Route, SharedAccountsRoute, ExactOutRoute instructions
- ✅ Volume tracking with `total_volume` and `total_swaps`

### Foundational Modules
- ✅ Integrated solana-common v0.3.1 foundational modules
- ✅ Block-level filtering via `blockFilter` for 75% data reduction
- ✅ Only processes blocks containing Jupiter transactions

### New Proto Fields
- ✅ `TradingData`: amount_in, amount_out, input_mint, output_mint, user_wallet
- ✅ `JupiterAnalytics`: total_volume, total_swaps
- ✅ `ProgramStat`: total_volume per program

## 📚 Documentation

- [Substreams Documentation](https://docs.substreams.dev/)
- [Foundational Stores Guide](https://docs.substreams.dev/how-to-guides/composing-substreams/foundational-stores/)
- [Jupiter Developer Docs](https://docs.jup.ag/)
- [Solana Documentation](https://docs.solana.com/)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Substreams Package**: https://substreams.dev/packages/jupiter-dex-substreams/v0.3.3
- **GitHub Repository**: https://github.com/PaulieB14/Jupiter-Dex-Substreams
- **Jupiter Website**: https://jup.ag
- **Solana Website**: https://solana.com

## 📞 Support

For support, please:
- Open an issue on [GitHub](https://github.com/PaulieB14/Jupiter-Dex-Substreams/issues)
- Join the [Substreams Discord](https://discord.gg/streamingfast)
- Check the [documentation](./docs)

---

**Built with ❤️ for the Jupiter and Solana ecosystem**