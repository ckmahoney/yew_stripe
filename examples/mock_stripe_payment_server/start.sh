#!/usr/bin/env bash
##
## start.sh — launch mock Stripe server (dev) + optional cleanup
##
## **No files are created by this script itself.** Any build artifacts
## (the `target/` directory) are removed right after the server stops,
## unless `--retain` is used.
##
## Usage:
##   ./start.sh [SK_TEST] [--retain] [--help]
##
## Positional args:
##   SK_TEST    Your Stripe secret key (e.g. sk_test_…). If omitted, the script will:
##                1. Look for a file named `sk_test` in this directory and load its contents.
##                2. If there’s a `.env` file, read only the `sk_test` key from it and override.
##   --retain   Skip removal of `target/` after exit
##   --help     Show this help message and exit
##

set -euo pipefail

#—— FUNCTIONS ——#

usage() {
  sed -n '1,20p' "$0"
  exit 0
}

cleanup() {
  if [[ "$RETAIN" == false ]]; then
    echo "🧹 Cleaning build artifacts…"
    rm -rf target
  else
    echo "⚠️  Retaining build artifacts."
  fi
}

#—— ARGUMENT PARSING & KEY RESOLUTION ——#

RETAIN=false
SK_TEST=""

# Extract flags & positional key
for arg in "$@"; do
  case "$arg" in
    --retain) RETAIN=true; shift ;;
    --help)   usage ;;
    *) 
      if [[ -z "$SK_TEST" ]]; then
        SK_TEST="$arg"
        shift
      else
        echo "Unknown argument: $arg" >&2
        usage
      fi
      ;;
  esac
done

# 1) Positional? else 2) sk_test file? else 3) .env? else fail
if [[ -z "$SK_TEST" ]]; then
  if [[ -f sk_test ]]; then
    SK_TEST="$(< sk_test)"
    SK_TEST="${SK_TEST//[$'\r\n']}"
  elif [[ -f .env ]]; then
    val="$(grep -E '^sk_test=' .env | tail -n1 | cut -d'=' -f2-)"
    SK_TEST="${val:-}"
  fi
fi

if [[ -z "$SK_TEST" ]]; then
  echo "Error: Stripe key not provided." >&2
  echo "Provide it as first arg, or in 'sk_test' file, or in .env (sk_test=…)." >&2
  exit 1
fi

export STRIPE_SECRET_KEY="$SK_TEST"

#—— SETUP TRAP ——#
trap cleanup EXIT

#—— MAIN ——#
echo "🚀 Starting mock Stripe server (dev build)…"
cargo run
# On exit—whether normal, error, or CTRL-C—the trap will remove target/
