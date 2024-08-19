# Admin

## 准备yaml配置
```bash
# 生成默认配置
kubeadm config print init-defaults > default-config.yaml
# 下载插件yaml
https://github.com/projectcalico/calico
curl https://raw.githubusercontent.com/projectcalico/calico/v3.28.1/manifests/calico.yaml -O
https://github.com/kubernetes-sigs/metrics-server/
wget https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
# 修改metrics-server的yaml,添加tls
spec:
  template:
    spec:
      containers:
      - args:
        - --kubelet-insecure-tls
```

## 初始化与安装clico与metrics插件
```bash
# 去主节点机器初始化集群
kubeadm init --config=/etc/kubernetes/init-control-plane.yml
kubectl apply -f /etc/kubernetes/calico/calico.yaml
kubectl apply -f /etc/kubernetes/metrics-server/components.yaml

# 去除主节点不支持调度污点
kubectl taint nodes [node-name] node-role.kubernetes.io/control-plane-

# 为用户添加客户端配置
mkdir -p $HOME/.kube
sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config
sudo chown $(id -u):$(id -g) $HOME/.kube/config

```
## Node加入集群
```bash
# 创建一个token
kubeadm token create --print-join-command
# 查看组装的token
kubeadm token list
# 获取sha256
openssl x509 -pubkey -in /etc/kubernetes/pki/ca.crt | \
  openssl rsa -pubin -outform der | \
  openssl dgst -sha256 -hex | sed 's/^.* //' | awk '{print "sha256:" $0}'
# 示例：加入集群
systemctl enable --now kubelet
kubeadm join node:6443 --token xxxx.xxxxxxxxxxxx \
    --discovery-token-ca-cert-hash sha256:xxxx \
    --control-plane 
```