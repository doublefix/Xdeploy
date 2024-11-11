## Manually uninstall nerdctl context

```bash
# 下载二进制文件
wget -P roles/nerdctl/release/assets/ https://github.com/containerd/nerdctl/releases/download/v1.7.6/nerdctl-full-1.7.6-linux-amd64.tar.gz
```

手动移除 containerd 的容器镜像

```bash
# 停止所有容器
nerdctl stop $(nerdctl ps -q)
nerdctl rm $(nerdctl ps -q -a)
# 删除所有镜像
nerdctl rmi -f $(nerdctl images -q)
# 停服务
sudo systemctl status containerd
sudo systemctl status buildkit
sudo systemctl status stargz-snapshotter
sudo systemctl stop containerd
sudo systemctl stop buildkit
sudo systemctl stop stargz-snapshotter
ps -aux|grep containerd
# 移除二进制
sudo rm /etc/systemd/system/containerd.service
sudo rm /etc/systemd/system/containerd.socket
# 删数据
sudo rm -rf /var/lib/containerd
sudo rm -rf /var/lib/buildkit
sudo rm -rf /run/containerd
sudo rm -rf /var/lib/containerd-stargz-grpc
sudo rm -rf .local/share/* # 普通用户
# 删除配置文件
sudo rm /etc/containerd/config.toml
# 删插件
sudo rm -rf /opt/containerd
# 重载
sudo systemctl daemon-reload
# 最后重启
```
