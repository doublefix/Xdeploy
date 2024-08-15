- 禁用swap
- 确保每个节点上 MAC 地址和 product_uuid 的唯一性
- 你可以使用命令 ip link 或 ifconfig -a 来获取网络接口的 MAC 地址
- 可以使用 sudo cat /sys/class/dmi/id/product_uuid 命令对 product_uuid 校验

新增内核模块
```bash
# 必须配置，overlay支持容器文件系统，br_netfilter管理网络流量的过滤
cat <<EOF | sudo tee /etc/modules-load.d/kube-modules.conf
overlay
br_netfilter
EOF

# 手动加载指定的模块，而无需重启系统
lsmod | grep br_netfilter
lsmod | grep overlay

```

iptable网络配置
```bash
# net.ipv4.ip_forward 是必须配置
# 控制桥接流量的 iptables 处理
# net.ipv4.ip_forward 启用 IP 转发，支持跨节点网络通信
cat <<EOF | sudo tee /etc/sysctl.d/kube-sysctl.conf
net.bridge.bridge-nf-call-iptables  = 1
net.bridge.bridge-nf-call-ip6tables = 1
net.ipv4.ip_forward                 = 1
EOF

# 立即应用这些内核参数而不重新启动
sudo sysctl --system

# 验证，保证全为1
sudo sysctl net.bridge.bridge-nf-call-iptables
sudo sysctl net.bridge.bridge-nf-call-ip6tables
sudo sysctl net.ipv4.ip_forward
```
