#!/bin/bash

# Define the location of the .env file (change if needed)
ENV_FILE="./auth-service/.env"

# Check if the .env file exists
if ! [[ -f "$ENV_FILE" ]]; then
  echo "Error: .env file not found!"
  exit 1
fi

# Read each line in the .env file (ignoring comments)
while IFS= read -r line; do
  # Skip blank lines and lines starting with #
  if [[ -n "$line" ]] && [[ "$line" != \#* ]]; then
    # Split the line into key and value
    key=$(echo "$line" | cut -d '=' -f1)
    value=$(echo "$line" | cut -d '=' -f2-)
    # Export the variable
    export "$key=$value"
  fi
done < <(grep -v '^#' "$ENV_FILE")

# Set defaults for local development
export AUTH_SERVICE_IP=""  # Empty for local - not needed
export NGINX_CONFIG="nginx-local.conf"

# Use all available CPU cores for Rust builds
export CARGO_BUILD_JOBS=$(nproc)

# Run with local override
docker compose -f compose.yml -f compose-local.yml build --parallel
docker compose -f compose.yml -f compose-local.yml up