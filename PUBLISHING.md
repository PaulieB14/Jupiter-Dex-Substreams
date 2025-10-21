# Publishing Jupiter DEX Substreams with Foundational Stores

This guide explains how to publish your Jupiter DEX Substreams with foundational store integration to substreams.dev.

## Prerequisites

1. **Substreams CLI**: Install the latest version
   ```bash
   npm install -g @substreams/cli
   ```

2. **Authentication**: Login to substreams.dev
   ```bash
   substreams login
   ```

## Publishing Process

### 1. Build and Test Locally

First, ensure your substreams builds correctly:

```bash
# Build the substreams
substreams build

# Test locally (optional)
substreams run map_jupiter_instructions --start-block 31310775 --stop-block +10
```

### 2. Publish to substreams.dev

Publish your substreams with the new foundational store integration:

```bash
# Publish the main substreams
substreams publish substreams.yaml

# Or publish the substreams.dev specific version
substreams publish substreams.dev.yaml
```

### 3. Verify Publication

Check that your substreams are available:

```bash
# List your published substreams
substreams list

# Check specific substreams
substreams info jupiter-dex-substreams@v0.2.0
```

## Foundational Store Dependencies

Your substreams now depend on these foundational stores:

- `spl-initialized-account@v0.1.2` - For SPL account ownership
- `jupiter-trading-data@v0.1.0` - For trading data aggregation
- `token-prices@v0.1.0` - For token price tracking

## Usage by Consumers

Other developers can now use your substreams with foundational stores:

```yaml
specVersion: v0.1.0
package:
  name: my-jupiter-analytics
  version: v0.1.0

imports:
  jupiter_dex: jupiter-dex-substreams@v0.2.0

modules:
  - name: my_analytics
    kind: map
    inputs:
      - map: jupiter_dex:map_jupiter_instructions
      - foundational-store: jupiter_dex:jupiter-trading-data@v0.1.0
      - foundational-store: jupiter_dex:token-prices@v0.1.0
```

## Version Management

- **Current Version**: v0.2.0 (with foundational stores)
- **Previous Version**: v0.1.0 (without foundational stores)

When publishing updates:
1. Increment the version number in `Cargo.toml`
2. Update the version in `substreams.yaml` and `substreams.dev.yaml`
3. Publish the new version

## Troubleshooting

### Common Issues

1. **Build Errors**: Ensure all dependencies are correctly specified in `Cargo.toml`
2. **Import Errors**: Verify that foundational store imports are correct
3. **Authentication**: Make sure you're logged in to substreams.dev

### Getting Help

- Check the [Substreams Documentation](https://docs.streamingfast.io/substreams)
- Join the [Substreams Discord](https://discord.gg/streamingfast)
- Review the [Foundational Stores Guide](FOUNDATIONAL_STORES.md)

## Next Steps

After publishing:

1. **Monitor Usage**: Track how your substreams are being used
2. **Gather Feedback**: Collect feedback from users
3. **Iterate**: Make improvements based on usage patterns
4. **Document**: Keep documentation updated with new features

## Benefits of Publishing

- **Discoverability**: Your substreams become discoverable on substreams.dev
- **Reusability**: Other developers can build on your work
- **Collaboration**: Enable community contributions
- **Versioning**: Proper version management for dependencies