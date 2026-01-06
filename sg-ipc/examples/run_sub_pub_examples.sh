#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -euo pipefail

temp_dir=$(mktemp -d)
echo "Temporary directory created at: $temp_dir"
socket_path="$temp_dir/pub_service.ipc"

log_dir="$SCRIPT_DIR/logs/pub_sub/"
rm -rf "$log_dir" || true
mkdir -p "$log_dir"
echo "Log directory created at: $log_dir"


cargo run --example pub_service -- "$socket_path" > "$log_dir/pub_server.log" &
pub_pid=$!

cargo run --example sub_service -- "$socket_path" > "$log_dir/sub_client0.log" &
sub0_pid=$!
cargo run --example sub_service -- "$socket_path" > "$log_dir/sub_client1.log" &
sub1_pid=$!

# Wait for publisher to finish
# Inspire by https://stackoverflow.com/questions/1058047/wait-for-a-process-to-finish/41613532#41613532
tail --pid=$pub_pid -f /dev/null


cleanup() {
    rm -rf "$temp_dir" || true
    kill $pub_pid $sub0_pid $sub1_pid 1>/dev/null 2>&1 || true
}

trap "cleanup" EXIT

compare_expected_log_to_file() {
  log_expected=$1
  log_file=$2
  log_actual=$(cat "$log_file")

  if [ "$log_actual" = "$log_expected" ]; then
      printf "\nLog matches expected output.\n"
  else
      printf "\nLog does not match expected output.\n"
      printf "Expected:\n%s\n" "$log_expected"
      printf "Actual (%s):\n%s\n" "$log_file" "$log_actual"
      exit 1
  fi
}

server_log_expected="Publishing message: Hello, World: 0
Publishing message: Hello, World: 1
Publishing message: Hello, World: 2
Publishing message: Hello, World: 3
Publishing message: Hello, World: 4
Publishing message: Hello, World: 5
Publishing message: Hello, World: 6
Publishing message: Hello, World: 7
Publishing message: Hello, World: 8
Publishing message: Hello, World: 9"

compare_expected_log_to_file "$server_log_expected" "$log_dir/pub_server.log"

clients_log_expected="Received message: Hello, World: 0

Received message: Hello, World: 1

Received message: Hello, World: 2

Received message: Hello, World: 3

Received message: Hello, World: 4

Received message: Hello, World: 5

Received message: Hello, World: 6

Received message: Hello, World: 7

Received message: Hello, World: 8

Received message: Hello, World: 9"

compare_expected_log_to_file "$clients_log_expected" "$log_dir/sub_client0.log"
compare_expected_log_to_file "$clients_log_expected" "$log_dir/sub_client1.log"
