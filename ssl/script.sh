sudo certbot --nginx -d yourdomain.com -d www.yourdomain.com



# 查看dns txt记录
nslookup -q=txt _acme-challenge.example.com
# 查看证书有效期
sudo openssl x509 -noout -dates -in /etc/letsencrypt/live/example.com/fullchain.pem
# 测试续签
sudo certbot renew --dry-run


# aliyun自动续签
crontab -e
1 1 */1 * * root certbot renew --manual --preferred-challenges dns --manual-auth-hook "alidns" --manual-cleanup-hook "alidns clean" --deploy-hook "nginx -s reload"
