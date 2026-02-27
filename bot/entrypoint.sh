#!/usr/bin/env bash
set -euo pipefail

show_help() {
  echo "Usage: $0 [--worker] [--bot] [--api] [--image-generation]"
  exit 1
}

worker=false
bot=false
api=false
image_generation=false

if [ $# -eq 0 ]; then
  show_help
fi

for arg in "$@"; do
  case "$arg" in
    --worker) worker=true ;;
    --bot)    bot=true ;;
    --api)    api=true ;;
    --image-generation) image_generation=true ;;
    -h|--help) show_help ;;
    *) echo "Unknown option: $arg"; show_help ;;
  esac
done

if [ "$worker" = true ]; then
  echo "Starting worker..."
  exec worker
fi

if [ "$api" = true ]; then
  echo "Starting API server..."
  exec api-server
fi

if [ "$image_generation" = true ]; then
  echo "Starting image generation worker..."
  exec image_generation
fi

if [ "$bot" = true ]; then
  echo "Starting bot..."
  exec kasuki
fi

# If we started things in background and didn't exec, wait for them
wait
