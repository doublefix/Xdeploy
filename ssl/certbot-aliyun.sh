#!/bin/bash

if [ -z "$2" ]; then
    echo "用法: $0 {request|renew} domain"
    exit 1
fi

ACTION="$1"
DOMAIN="$2"

# 自动认证和清理脚本路径
AUTH_HOOK="/path/to/alidns"
CLEANUP_HOOK="/path/to/alidns clean"

# 证书申请
request_certificate() {
    echo "申请证书..."
    sudo certbot certonly \
        --manual \
        --preferred-challenges dns-01 \
        --manual-auth-hook "$AUTH_HOOK" \
        --manual-cleanup-hook "$CLEANUP_HOOK" \
        --server https://acme-v02.api.letsencrypt.org/directory \
        -d "$DOMAIN"
}

# 证书续费
renew_certificate() {
    echo "续费证书..."
    sudo certbot renew \
        --manual \
        --preferred-challenges dns-01 \
        --manual-auth-hook "$AUTH_HOOK" \
        --manual-cleanup-hook "$CLEANUP_HOOK"
}

# 根据操作类型选择相应的函数
case "$ACTION" in
    request)
        request_certificate
        ;;
    renew)
        renew_certificate
        ;;
    *)
        echo "用法: $0 {request|renew} domain"
        exit 1
        ;;
esac

# ./manage_certificates.sh request "*.example.com"
# ./manage_certificates.sh renew "*.example.com"