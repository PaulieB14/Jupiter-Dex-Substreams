# Jupiter DEX Events Substream

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

## Jupiter Program IDs

- **Jupiter Swap v6**: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
- **Jupiter Swap v4/v3**: `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB`
- **Jupiter Swap v2**: `JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph`
- **Jupiter Swap v1**: `JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo`
- **Jupiter Limit Orders**: `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu`
- **Jupiter DCA**: `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M`

## Usage

### Run the substream:

```bash
# Install substreams CLI
curl -sSL https://substreams.dev/install.sh | bash

# Run Jupiter events
substreams run substream.yaml jupiter_events -e mainnet.sol.streamingfast.io:443 -s 325766951 -t +1

# Run specific event types
substreams run substream.yaml jupiter_swap_events -e mainnet.sol.streamingfast.io:443 -s 325766951 -t +1
substreams run substream.yaml jupiter_limit_order_events -e mainnet.sol.streamingfast.io:443 -s 325766951 -t +1
substreams run substream.yaml jupiter_dca_events -e mainnet.sol.streamingfast.io:443 -s 325766951 -t +1
```

### Build the substream:

```bash
# Build the Rust code
cargo build --release

# Generate protobuf bindings
substreams protogen
```

## Event Types

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

## Development

This substream is built using:
- **Rust** for the core logic
- **Protocol Buffers** for data serialization
- **Substreams Solana** for blockchain data access

## References

- [Jupiter GitHub](https://github.com/jup-ag)
- [Substreams Documentation](https://docs.substreams.dev/)
- [Jupiter Developer Docs](https://docs.jup.ag/)
