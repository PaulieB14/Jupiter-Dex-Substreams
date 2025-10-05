#!/bin/bash

echo "🚀 Building Jupiter Substream Package for Publishing..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build the Rust code
echo "🔨 Building Rust code..."
cargo build --release --target wasm32-unknown-unknown

# Generate protobuf bindings (if possible)
echo "📦 Generating protobuf bindings..."
substreams protogen substream.yaml || echo "⚠️  Protobuf generation failed - continuing..."

# Create the package
echo "📦 Creating Substreams package..."
substreams pack substream.yaml

echo "✅ Jupiter Substream package ready for publishing!"
echo "📁 Package file: jupiter-dex-events-v0.1.0.spkg"
echo ""
echo "To publish, run:"
echo "substreams publish jupiter-dex-events-v0.1.0.spkg"
