# Jupiter DEX Events Substream - Package Metadata

## Package Information
- **Name**: jupiter-dex-events
- **Version**: v0.1.0
- **Network**: Solana
- **Chain**: solana
- **Description**: Jupiter DEX aggregator events substream for Solana

## Jupiter Program IDs Tracked
- **Jupiter Swap v6**: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
- **Jupiter Swap v4/v3**: `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB`
- **Jupiter Swap v2**: `JUP3c2Uh3WA4Ng34tw6kPd2G4C5BB21Xo36Je1s32Ph`
- **Jupiter Swap v1**: `JUP2jxvXaqu7NQY1GmNF4m1vodw12LVXYxbFL2uJvfo`
- **Jupiter Limit Orders**: `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu`
- **Jupiter DCA**: `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M`

## Event Types
1. **Swap Events**: Route discovery, execution, slippage tracking
2. **Limit Order Events**: Order placement, execution, cancellation
3. **DCA Events**: Scheduled purchases, execution status
4. **Aggregation Events**: Cross-DEX routing decisions

## Usage
```bash
substreams run substream.yaml jupiter_events -e mainnet.sol.streamingfast.io:443 -s 325766951 -t +1
```

## Dependencies
- Solana Substreams v0.2.3
- Rust WASM target
- Protocol Buffers

## Author
Jupiter Substream Team

## License
MIT
