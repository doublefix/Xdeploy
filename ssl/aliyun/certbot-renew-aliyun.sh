#!/bin/bash

# Ensure the script is run with sudo privileges
if [ "$(id -u)" -ne "0" ]; then
  echo "This script must be run as root or with sudo." >&2
  exit 1
fi

# Check if the required number of arguments are provided
if [ "$#" -gt 1 ]; then
  echo "Usage: $0 [--dry-run]"
  exit 1
fi

DRY_RUN=""
if [ "$#" -eq 1 ] && [ "$1" == "--dry-run" ]; then
  DRY_RUN="--dry-run"
fi

sudo certbot renew \
    --manual \
    --preferred-challenges dns \
    --manual-auth-hook "alidns" \
    --manual-cleanup-hook "alidns clean" \
    $DRY_RUN

if [ $? -eq 0 ]; then
  echo "Certbot renewal command completed successfully."
else
  echo "Certbot renewal command failed." >&2
  exit 1
fi
