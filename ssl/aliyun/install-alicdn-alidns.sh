#!/bin/bash

sudo wget https://aliyuncli.alicdn.com/aliyun-cli-linux-latest-amd64.tgz
sudo tar xzvf aliyun-cli-linux-latest-amd64.tgz
sudo cp aliyun /usr/local/bin


sudo wget https://cdn.jsdelivr.net/gh/justjavac/certbot-dns-aliyun@main/alidns.sh
sudo cp alidns.sh /usr/local/bin
sudo chmod +x /usr/local/bin/alidns.sh
sudo ln -s /usr/local/bin/alidns.sh /usr/local/bin/alidns


sudo rm aliyun-cli-linux-latest-amd64.tgz
sudo rm aliyun
sudo rm alidns.sh