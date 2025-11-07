#!/bin/bash

set -e  # Exit on any error

echo "🧹 Cleaning previous builds..."
rm -rf npx-cli/dist
mkdir -p npx-cli/dist/macos-arm64

echo "🔨 Building frontend..."
(cd frontend && npm run build)

echo "🔨 Building Rust binaries..."
cargo build --release --manifest-path Cargo.toml
cargo build --release --bin mcp_task_server --manifest-path Cargo.toml

echo "📦 Creating distribution package..."

# Copy the main binary
cp target/release/server anyon
zip -q anyon.zip anyon
rm -f anyon 
mv anyon.zip npx-cli/dist/macos-arm64/anyon.zip

# Copy the MCP binary
cp target/release/mcp_task_server anyon-mcp
zip -q anyon-mcp.zip anyon-mcp
rm -f anyon-mcp
mv anyon-mcp.zip npx-cli/dist/macos-arm64/anyon-mcp.zip

echo "✅ NPM package ready!"
echo "📁 Files created:"
echo "   - npx-cli/dist/macos-arm64/anyon.zip"
echo "   - npx-cli/dist/macos-arm64/anyon-mcp.zip"
