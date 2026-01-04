#!/usr/bin/env bash
set -euo pipefail

show_help() {
  echo "Usage: $0 [--worker] [--bot] [--api]"
  exit 1
}

worker=false
bot=false
api=false

if [ $# -eq 0 ]; then
  show_help
fi

for arg in "$@"; do
  case "$arg" in
    --worker) worker=true ;;
    --bot)    bot=true ;;
    --api)    api=true ;;
    -h|--help) show_help ;;
    *) echo "Unknown option: $arg"; show_help ;;
  esac
done

if [ "$bot" = true ]; then
  echo "Running migrations..."
  migration
fi

if [ "$worker" = true ]; then
  echo "Starting worker..."
  if [ "$bot" = true ] || [ "$api" = true ]; then
    worker &
  else
    exec worker
  fi
fi

if [ "$api" = true ]; then
  echo "Starting API server..."
  if [ "$bot" = true ]; then
    api-server &
  else
    exec api-server
  fi
fi

if [ "$bot" = true ]; then
  echo "Starting bot..."
  exec kasuki
fi

# If we started things in background and didn't exec, wait for them
wait
