# Upgrading to v0.3.2

This guide helps you upgrade from v0.3.1 to v0.3.2 with foundational store integration.

## What's New

### Performance Improvements
- **75% Data Reduction**: Excludes voting transactions automatically
- **3-5x Faster Queries**: Shared caching via foundational stores
- **Block Filtering**: Processes only Jupiter-related blocks
- **Lower Costs**: Reduced infrastructure expenses

### New Modules
- `store_swap_volumes` - Track trading volumes by token pair
- `store_unique_traders` - Monitor unique wallet addresses  

### Architecture Changes
- New base module: `jupiter_filtered_transactions`
- Integrated `solana-common` v0.3.0 foundational modules
- Block-level filtering for efficiency

## Breaking Changes

### None! 
v0.3.2 maintains **full backward compatibility** with v0.3.1 output formats.

## Migration Steps

### If Using the Package Directly

Simply update your substreams reference:

```bash
# Old (v0.3.1)
substreams run https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.3.1/jupiter-dex-substreams-v0.3.1.spkg

# New (v0.3.2)
substreams run https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.3.2/jupiter-dex-substreams-v0.3.2.spkg
```

### If Building from Source

```bash
# Pull latest changes
git fetch origin
git checkout v0.3.2

# Build
substreams build

# Package
substreams pack
```

### If Importing as Dependency

Update your `substreams.yaml`:

```yaml
imports:
  jupiter: https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.3.2/jupiter-dex-substreams-v0.3.2.spkg
```

## Testing Your Upgrade

### 1. Validate Output Format

```bash
# Test that outputs match expected format
substreams run substreams.yaml map_jupiter_instructions \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1
```

### 2. Compare Performance

Run the same query range on both versions:

```bash
# v0.3.1
time substreams run v0.3.1.spkg map_jupiter_instructions -s 325766951 -t +1000

# v0.3.2  
time substreams run v0.3.2.spkg map_jupiter_instructions -s 325766951 -t +1000
```

You should see ~3-5x improvement in v0.3.2.

### 3. Verify Data Accuracy

The output data should be identical, just delivered faster:

```bash
# Export v0.3.1 results
substreams run v0.3.1.spkg map_jupiter_analytics -s 325766951 -t +100 > v0.3.1.json

# Export v0.3.2 results
substreams run v0.3.2.spkg map_jupiter_analytics -s 325766951 -t +100 > v0.3.2.json

# Compare (should match)
diff v0.3.1.json v0.3.2.json
```

## New Capabilities

### Access Swap Volume Store

```bash
substreams run substreams.yaml store_swap_volumes \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1000
```

### Monitor Unique Traders

```bash
substreams run substreams.yaml store_unique_traders \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1000
```

### Leverage Block Filtering

The new architecture automatically filters blocks, but you can customize:

```yaml
# In your substreams.yaml
params:
  jupiter_programs: >
    program:JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4 ||
    program:YOUR_CUSTOM_PROGRAM_ID
```

## Rollback Plan

If you encounter issues, you can easily rollback:

```bash
# Revert to v0.3.1
git checkout v0.3.1
substreams build
substreams pack
```

Or reference the v0.3.1 package directly:

```bash
substreams run https://github.com/PaulieB14/Jupiter-Dex-Substreams/releases/download/v0.3.1/jupiter-dex-substreams-v0.3.1.spkg
```

## Support

If you encounter any issues during upgrade:

1. Check the [CHANGELOG](CHANGELOG.md) for detailed changes
2. Review [GitHub Issues](https://github.com/PaulieB14/Jupiter-Dex-Substreams/issues)
3. Open a new issue with:
   - Version you're upgrading from
   - Error messages or unexpected behavior
   - Steps to reproduce

## Performance Benchmarks

Expected improvements in v0.3.2:

| Metric | v0.3.1 | v0.3.2 | Improvement |
|--------|--------|--------|-------------|
| Data Processed | 100% | 25% | 75% reduction |
| Query Speed | 1x | 3-5x | 3-5x faster |
| Infrastructure Cost | 1x | ~0.3x | ~70% savings |
| Block Processing | All blocks | Filtered | Skip irrelevant |

## What Happens Under the Hood

### Old Architecture (v0.3.1)
```
Raw Solana Blocks (100% data)
    ↓
Process ALL blocks
    ↓
Extract Jupiter events
```

### New Architecture (v0.3.2)
```
Solana Common Foundational Store (25% data, no votes)
    ↓
Block Filter (Jupiter programs only)
    ↓
Process ONLY relevant blocks
    ↓
Extract Jupiter events
```

## Next Steps

After upgrading:

1. ✅ Monitor performance improvements
2. ✅ Explore new store modules
3. ✅ Consider integrating swap volume analytics
4. ✅ Share feedback or improvements via PR

Happy indexing! 🚀