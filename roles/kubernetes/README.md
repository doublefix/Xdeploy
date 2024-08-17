```bash
# 下载预安装文件
bash roles/kubernetes/scripts/download_bin.sh
bash roles/kubernetes/scripts/download_conf.sh
bash roles/kubernetes/scripts/clean.sh

# 安装
ansible-playbook playbooks/kubernetes.yml
# 卸载
ansible-playbook playbooks/clean/kubernetes.yml
```

## 注意事项

Redhat 将 SELinux 设置为 permissive 模式（相当于将其禁用）

```bash
sudo setenforce 0sudo sed -i 's/^SELINUX=enforcing$/SELINUX=permissive/' /etc/selinux/config
```

## Manually uninstall k8s

手动卸载通过包管理器安装的 k8s

```bash
# 从系统剔除
kubectl drain [nodename] --delete-local-data --force --ignore-daemonsets
kubectl delete node [nodename]
# 初始化节点，切断与k8s的联系
sudo kubeadm reset
sudo rm -rf /etc/cni/net.d
# 重置网络
sudo iptables -F
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo ipvsadm --clear
# 停止服务
sudo systemctl status kubelet
sudo systemctl stop kubelet
sudo systemctl disable kubelet
```

### Redhat 系

```bash
# 卸载 Kubernetes 组件
sudo yum remove -y kubelet kubeadm kubectl
# 删除k8s仓库
sudo rm -f /etc/yum.repos.d/kubernetes.repo
# 删除k8s数据
sudo rm -rf /etc/kubernetes
sudo rm -rf /var/lib/kubelet
# 解除版本锁定
sudo yum versionlock delete kubelet kubeadm kubectl
# 检查清理
sudo find / -name '*kube*'
sudo find / -name '*kube*' -exec rm -rf {} +
# 清除安装缓存
sudo rm -rf /var/cache/dnf/kubernetes*
sudo rm -rf /var/cache/dnf/kubernetes.solv
sudo rm -rf /var/cache/dnf/kubernetes-filenames.solvx
# 杂项配置
sudo rm -f /usr/lib/systemd/system/podman-kube@.service
sudo rm -f /usr/lib/systemd/user/podman-kube@.service
sudo rm -f /usr/lib/firewalld/services/kube-*
sudo rm -rf /usr/libexec/kubernetes
sudo rm -rf /usr/share/man/man1/podman-kube*.1.gz
sudo rm -rf /usr/share/cockpit/branding/kubernetes
```

### Debian 系

```bash
# 解锁
sudo apt-mark unhold kubelet kubeadm kubectl
# 卸载
sudo apt-get purge -y kubelet kubeadm kubectl
# 清理缓存
sudo apt-get autoremove -y
sudo apt-get autoclean
# 删除仓库
sudo rm /etc/apt/sources.list.d/kubernetes.list
sudo rm /etc/apt/keyrings/kubernetes-apt-keyring.gpg
# 清理数据
sudo rm -rf /etc/kubernetes
sudo rm -rf /var/lib/kubelet
sudo rm -rf /var/lib/etcd
sudo rm -rf ~/.kube
```
