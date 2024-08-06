#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 example.com"
    exit 1
fi

DOMAIN=$1

echo "Certbot version:"
sudo certbot --version

echo "Requesting certificate for *.$DOMAIN"
sudo certbot certonly \
    --manual \
    --preferred-challenges dns-01 \
    --server https://acme-v02.api.letsencrypt.org/directory \
    -d "*.$DOMAIN"

# 添加云解析DNS的TXT记录:
# Key _acme-challenge.example.com
# Value system print

sudo ls /etc/letsencrypt/live

# 手动脚本，不适合自动续订


