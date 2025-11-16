#!/bin/bash
# Test script for both providers

echo "=== Testing Echo Provider ==="
echo "Default (should be Echo):"
cargo build --quiet
echo "Run with: cargo run"
echo ""

echo "=== Testing Shell Provider ==="
echo "Set PROVIDER=shell to use Shell provider:"
echo "Run with: PROVIDER=shell cargo run"
echo ""

echo "=== Example Usage ==="
echo "Echo provider: type 'hello world', press Enter"
echo "Shell provider: type 'echo hello', press Enter, or 'ls -la'"
echo ""
echo "Both: press 'q' to quit"
