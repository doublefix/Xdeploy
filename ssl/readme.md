# TLS certificate application

## Certbot

```bash
# Install certbot
sudo bash ssl/install-certbot.sh

# 手动请求tls证书
sudo bash ssl/certbot-manual.sh example.com

# 手动验证dns的txt记录内容
nslookup -q=txt _acme-challenge.example.com

# 查看证书有效期
sudo openssl x509 -noout -dates -in /etc/letsencrypt/live/example.com/fullchain.pem

# 测试续签(need --manual-auth-hook auth-hook.sh)
sudo certbot renew --dry-run

```

## Aliyun

每个 dns 厂商的添加 dns 的 txt 记录不同，导致申请方式不同

```bash
# Install alicdn and alidns script
sudo bash ssl/aliyun/install-aliyuncli-alidns.sh

# 配置本地AK，需要从aliyun官网拿到AK
sudo bash ssl/aliyun/conf-aliyun.sh [YourAccessKeyId] [YourAccessKeySecret]

# 手动申请证书
sudo bash ssl/aliyun/certbot-manual-aliyun.sh example.com --dry-run

# 自动更新证书
sudo bash ssl/aliyun/certbot-renew-aliyun.sh --dry-run

# aliyun自动续签
crontab -e
1 1 */1 * * root certbot renew --manual --preferred-challenges dns --manual-auth-hook "alidns" --manual-cleanup-hook "alidns clean" --deploy-hook "nginx -s reload"

```
