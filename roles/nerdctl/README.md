```bash
# Download
bash roles/nerdctl/scripts/download_nerdctl.sh

# Install
ansible-playbook playbooks/nerdctl.yml
ansible-playbook playbooks/nerdctl.yml -e "operation=install"

# Uninstall
ansible-playbook playbooks/nerdctl.yml -e "operation=uninstall"
```

## Manually uninstall nerdctl context

手动移除 k8s 的容器镜像

```bash
# 停止所有容器
nerdctl stop $(nerdctl ps -q)
nerdctl rm $(nerdctl ps -q -a)
# 删除所有镜像
nerdctl rmi -f $(nerdctl images -q)
# 停服务
sudo systemctl status containerd
sudo systemctl stop containerd
ps aux | grep containerd
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
```
