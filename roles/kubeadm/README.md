# Admin

## 准备yaml配置
```bash
https://github.com/projectcalico/calico
curl https://raw.githubusercontent.com/projectcalico/calico/v3.28.1/manifests/calico.yaml -O
https://github.com/kubernetes-sigs/metrics-server/
wget https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

```

## 初始化与安装clico与metrics插件
```bash
# 生成默认配置
kubeadm config print init-defaults > default-config.yaml
# 去主节点机器初始化集群
kubeadm init --config=/etc/kubernetes/init-control-plane.yml
kubeadm apply -f /etc/kubernetes/calico/calico.yaml
kubeadm apply -f /etc/kubernetes/metrics-server/components.yaml

```