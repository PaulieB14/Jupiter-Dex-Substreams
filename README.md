# Jupiter DEX Events Substream

[![Substreams](https://img.shields.io/badge/Substreams-v0.1.2-blue)](https://substreams.dev/packages/jupiter-dex-events/v0.1.2)
[![Solana](https://img.shields.io/badge/Network-Solana-purple)](https://solana.com)
[![Jupiter](https://img.shields.io/badge/DEX-Jupiter-orange)](https://jup.ag)

A comprehensive Substreams package for tracking Jupiter DEX aggregator events on Solana blockchain.

## 🌟 Overview

This substream provides complete visibility into Jupiter's DEX aggregation ecosystem, tracking:

- **🔄 Jupiter Swap Events** (v1-v6): Token swap aggregations across multiple DEXs
- **📋 Jupiter Limit Orders**: Advanced order management functionality  
- **💰 Jupiter DCA**: Dollar Cost Averaging automation
- **🧠 Aggregation Events**: Cross-DEX routing decisions and arbitrage opportunities

## 🚀 Key Features

- **Multi-Version Support**: Tracks Jupiter v1-v6 simultaneously
- **Cross-DEX Analysis**: Understands routing across Raydium, Orca, and other DEXs
- **Advanced Features**: Limit orders, DCA, and aggregation logic
- **Real-time Events**: Live tracking of Jupiter's aggregation decisions

## 📦 Installation

```bash
# Install Substreams CLI
curl -sSL https://substreams.dev/install.sh | bash

# Run Jupiter events
substreams run jupiter-dex-events@v0.1.2 jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1
```

## 🏗️ Development

### Prerequisites

- Rust 1.70+
- Substreams CLI
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

# Run a module locally (e.g. enriched Jupiter instructions)
substreams run substreams.yaml map_jupiter_instructions \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1
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
substreams gui jupiter-dex-events@v0.1.2 jupiter_events

# Run specific block range
substreams run jupiter-dex-events@v0.1.2 jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
```

### Advanced Usage
```bash
# Run with custom headers
substreams run jupiter-dex-events@v0.1.2 jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -H "X-Substreams-Parallel-Workers: 20" \
  -s 325766951 -t +1
```

## 🏗️ Architecture

This substream is built using:

- **Rust** for the core logic
- **Protocol Buffers** for data serialization  
- **Substreams Solana** for blockchain data access
- **Jupiter SDK** integration for event parsing

## 📚 Documentation

- [Substreams Documentation](https://docs.substreams.dev/)
- [Jupiter Developer Docs](https://docs.jup.ag/)
- [Solana Documentation](https://docs.solana.com/)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Substreams Package**: https://substreams.dev/packages/jupiter-dex-events/v0.1.2
- **GitHub Repository**: https://github.com/PaulieB14/Jupiter-Dex-Substreams
- **Jupiter Website**: https://jup.ag
- **Solana Website**: https://solana.com

## 📞 Support

For support, please open an issue on GitHub or contact us through the Substreams community.

---

**Built with ❤️ for the Jupiter and Solana ecosystem**