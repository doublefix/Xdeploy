#!/bin/bash

if [ "$(id -u)" -ne "0" ]; then
  echo "This script must be run as root or with sudo." >&2
  exit 1
fi

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <domain> [--dry-run]"
  exit 1
fi

DOMAIN=$1

DRY_RUN=""
if [ "$#" -eq 2 ] && [ "$2" == "--dry-run" ]; then
  DRY_RUN="--dry-run"
fi

sudo certbot certonly \
    --manual --preferred-challenges dns \
    --manual-auth-hook "alidns" \
    --manual-cleanup-hook "alidns clean" \
    -d "*.$DOMAIN" \
    $DRY_RUN

if [ $? -eq 0 ]; then
  echo "Certbot command completed successfully."
else
  echo "Certbot command failed." >&2
  exit 1
fi
