#!/bin/bash

if [ "$(id -u)" -ne "0" ]; then
  echo "This script must be run as root or with sudo." >&2
  exit 1
fi

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  echo "Usage: $0 <domain> <cert-type> [--dry-run]"
  echo "cert-type: 'wildcard' for *.<domain> or 'root' for <domain>"
  exit 1
fi

DOMAIN=$1
CERT_TYPE=$2

if [ "$CERT_TYPE" != "wildcard" ] && [ "$CERT_TYPE" != "root" ]; then
  echo "Invalid cert-type. Use 'wildcard' for *.<domain> or 'root' for <domain>."
  exit 1
fi

DRY_RUN=""
if [ "$#" -eq 3 ] && [ "$3" == "--dry-run" ]; then
  DRY_RUN="--dry-run"
fi

if [ "$CERT_TYPE" == "wildcard" ]; then
  DOMAIN_ARG="-d *.$DOMAIN"
else
  DOMAIN_ARG="-d $DOMAIN"
fi

sudo certbot certonly \
    --manual --preferred-challenges dns \
    --manual-auth-hook "alidns" \
    --manual-cleanup-hook "alidns clean" \
    $DOMAIN_ARG \
    $DRY_RUN

if [ $? -eq 0 ]; then
  echo "Certbot command completed successfully."
else
  echo "Certbot command failed." >&2
  exit 1
fi
