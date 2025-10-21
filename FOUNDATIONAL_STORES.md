# Foundational Stores Integration for Jupiter DEX Substreams

This document explains how Foundational Stores are integrated into the Jupiter DEX Substreams project to provide high-performance, fork-aware data storage and retrieval.

## Overview

Foundational Stores provide a unified interface for ingesting, storing, and serving time-series blockchain data with fork-awareness capabilities. This integration enables:

- **Fork-Aware Operations**: Automatic handling of blockchain reorganizations with rollback capabilities
- **Real-time Ingestion**: Continuous processing of streaming Substreams output
- **High-Performance Serving**: Optimized gRPC API for fast data retrieval
- **Block-level Versioning**: Every piece of data is tagged with the block number where it originated

## Architecture

The integration consists of three main components:

### 1. SPL Initialized Account Store
**Purpose**: Track SPL token account ownership for resolving transfer instructions.

**Key Features**:
- Maps SPL token accounts to their owners
- Tracks mint associations
- Handles InitializeAccount, InitializeAccount2, and InitializeAccount3 instructions

**Usage**:
```rust
use substreams::store::FoundationalStore;

#[substreams::handlers::map]
fn map_jupiter_instructions(
    transactions: Transactions,
    account_owner_store: FoundationalStore,
) -> Result<JupiterInstructions, Error> {
    // Query account owners
    let response = account_owner_store.get_all(&account_keys);
    // Process responses...
}
```

### 2. Jupiter Trading Data Store
**Purpose**: Aggregate and store Jupiter trading data for analytics.

**Key Features**:
- Tracks trading volumes
- Stores trade metadata
- Enables historical analysis

### 3. Token Price Store
**Purpose**: Track token prices and price movements over time.

**Key Features**:
- Real-time price tracking
- Volume aggregation
- Price change calculations

### 4. Jupiter Analytics Store
**Purpose**: Provide comprehensive analytics using data from all foundational stores.

**Key Features**:
- 24-hour volume tracking
- Top token analysis
- User activity patterns
- Price movement analysis

## Data Flow

```
Solana Transactions
        ↓
SPL Account Store (Producer)
        ↓
Jupiter Trading Store (Producer)
        ↓
Token Price Store (Producer)
        ↓
Jupiter Instructions (Consumer) ← Uses all stores
        ↓
Jupiter Analytics (Consumer) ← Uses all stores
```

## Configuration

The foundational stores are configured in `substreams.yaml`:

```yaml
modules:
  # Producer modules (create foundational store entries)
  - name: map_spl_initialized_account
    kind: map
    inputs:
      - map: solana_common:transactions_by_programid_without_votes
    output:
      type: proto:sf.substreams.foundational_store.v1.Entries

  # Consumer modules (use foundational stores)
  - name: map_jupiter_instructions
    kind: map
    inputs:
      - map: solana_common:transactions_by_programid_without_votes
      - foundational-store: spl-initialized-account@v0.1.2
      - foundational-store: jupiter-trading-data@v0.1.0
      - foundational-store: token-prices@v0.1.0
```

## Usage Examples

### Querying Account Owners
```rust
let account_keys: Vec<Vec<u8>> = transfer_accounts.iter()
    .map(|addr| bs58::decode(addr).into_vec().unwrap())
    .collect();

let response = account_owner_store.get_all(&account_keys);

for entry in response.entries {
    if entry.response.unwrap().response == ResponseCode::Found as i32 {
        let account_owner = AccountOwner::decode(
            entry.response.unwrap().value.unwrap().value.as_slice()
        )?;
        // Use account owner data...
    }
}
```

### Accessing Trading Data
```rust
let trading_key = format!("jupiter_trade_{}", hex::encode(&instruction.accounts[0]));
let response = trading_data_store.get(&trading_key.as_bytes().to_vec());
```

### Retrieving Token Prices
```rust
let price_key = format!("token_price_{}", hex::encode(&mint_address));
let response = token_price_store.get(&price_key.as_bytes().to_vec());
```

## Benefits

1. **Fork-Aware Data**: All data is automatically rolled back during blockchain reorganizations
2. **High Performance**: Optimized storage and retrieval for real-time applications
3. **Rich Analytics**: Comprehensive trading data and analytics capabilities
4. **Account Resolution**: Automatic resolution of SPL token account ownership
5. **Historical Data**: Access to historical trading data and price movements

## Response Codes

Foundational stores return detailed status information:

- `FOUND`: Key exists and value was retrieved successfully
- `NOT_FOUND`: Key does not exist at the requested block
- `NOT_FOUND_FINALIZE`: Key was deleted after finality (LIB) - historical reference
- `NOT_FOUND_BLOCK_NOT_REACHED`: Requested block has not been processed yet

## Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
substreams = "0.13"
substreams-solana = "0.13"
substreams-foundational-store = "0.1"
substreams-store = "0.13"
```

## Next Steps

1. **Deploy the Substreams**: Deploy your updated substreams with foundational store integration
2. **Set up Foundational Store Backend**: Configure your preferred storage backend (Badger or PostgreSQL)
3. **Monitor Performance**: Track the performance of your foundational stores
4. **Extend Functionality**: Add more specialized stores for your specific use cases

## Related Resources

- [Foundational Stores Overview](https://docs.streamingfast.io/substreams/foundational-stores)
- [SPL Initialized Account Store](https://docs.streamingfast.io/substreams/foundational-stores/spl-initialized-account)
- [Jupiter DEX Documentation](https://docs.jup.ag/)