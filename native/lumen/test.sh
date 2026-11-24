#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
sh "$HERE/host.sh" "cargo test --locked $*"
