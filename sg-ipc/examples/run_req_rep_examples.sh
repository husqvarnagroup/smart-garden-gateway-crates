#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -euo pipefail

temp_dir=$(mktemp -d)
echo "Temporary directory created at: $temp_dir"
socket_path="$temp_dir/rep_service.ipc"

log_dir="$SCRIPT_DIR/logs/req_rep/"
rm -rf "$log_dir" || true
mkdir -p "$log_dir"
echo "Log directory created at: $log_dir"

cargo run --example rep_service -- "$socket_path" > "$log_dir/server.log" &
server_pid=$!

cleanup() {
    rm -rf "$temp_dir"
    kill $server_pid
}

trap "cleanup" EXIT

# Wait for the server to bind to the socket
sleep 1

cargo run --example req_service -- "$socket_path" "Hello0" "Hello1" "Hello2" "Hello3" "Hello4" "Hello5" "Hello6" > "$log_dir/client0.log" &
cargo run --example req_service -- "$socket_path" "HelloA" "HelloB" "HelloC" "HelloD" "HelloE" > "$log_dir/client1.log" &

# give clients some time to shutdown
sleep 5

compare_expected_log_to_file() {
  log_expected=$1
  log_file=$2
  log_actual=$(cat "$log_file")

  if [ "$log_actual" = "$log_expected" ]; then
      printf "\nLog matches expected output."
  else
      printf "\nLog does not match expected output."
      printf "Expected:\n%s\n" "$log_expected"
      printf "Actual (%s):\n%s\n" "$log_file" "$log_actual"
      exit 1
fi
}

# Print server log to check that client requests interleave.
# The log can not be diffed to an expected output as the order
# in the log is not deterministic.
printf "Server log actual:\n"
cat "$log_dir/server.log"

# Check that the log of client0 is as expected
client0_log_expected="Received reply: Reply from server for Hello0\n

Received reply: Reply from server for Hello1\n

Received reply: Reply from server for Hello2\n

Received reply: Reply from server for Hello3\n

Received reply: Reply from server for Hello4\n

Received reply: Reply from server for Hello5\n

Received reply: Reply from server for Hello6\n"

compare_expected_log_to_file "$client0_log_expected" "$log_dir/client0.log"


# Check that the log of client0 is as expected
client1_log_expected="Received reply: Reply from server for HelloA\n

Received reply: Reply from server for HelloB\n

Received reply: Reply from server for HelloC\n

Received reply: Reply from server for HelloD\n

Received reply: Reply from server for HelloE\n"

compare_expected_log_to_file "$client1_log_expected" "$log_dir/client1.log"
