#!/bin/bash
# Install a test example and check its output (traces).
set -eu -o pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

test_example_install_dir="$script_dir/test-example"
mkdir -p "$test_example_install_dir"

example_name="log_output"
cargo install --force --path "$script_dir/.." --example "$example_name" --root "$test_example_install_dir"

RUST_LOG=debug "$test_example_install_dir/bin/$example_name" > "$test_example_install_dir/output.log" 2>&1

if cmp -s "$script_dir/expected.log" "$test_example_install_dir/output.log"; then
    echo "Test passed: Output matches expected log."
else
    echo "Test failed: Output does not match expected log."
    diff "$script_dir/expected.log" "$test_example_install_dir/output.log"
    exit 1
fi
