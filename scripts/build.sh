#!/bin/bash
set -e

# Default environment
ENV="development"

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --env) ENV="$2"; shift ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

echo "Building for environment: $ENV"

# Load configuration
if [ -f "config/.env.$ENV" ]; then
    # Export variables from .env file, ignoring comments
    export $(grep -v '^#' "config/.env.$ENV" | xargs)
    echo "Loaded config/.env.$ENV"
else
    echo "Error: Configuration file config/.env.$ENV not found!"
    exit 1
fi

# Build command based on environment
if [ "$ENV" == "production" ]; then
    echo "Running release build..."
    cargo build --release --locked
elif [ "$ENV" == "staging" ]; then
    echo "Running release build with debug info..."
    # Enable debug symbols in release mode for staging
    RUSTFLAGS="-g" cargo build --release --locked
else
    echo "Running development build..."
    cargo build
fi

echo "Build complete for $ENV!"
