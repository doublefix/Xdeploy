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
systemctl enable --now kubelet
journalctl -u kubelet -f
sudo systemctl start kubelet
sudo systemctl stop kubelet
sudo systemctl status kubelet
# 手动初始化集群
sudo kubeadm reset
rm -rf $HOME/.kube
kubeadm init --config=/etc/kubernetes/init-control-plane.yml
sudo kubeadm init \
    --apiserver-advertise-address=10.187.6.4 \
    --control-plane-endpoint=ubuntu \
    --kubernetes-version v1.31.0 \
    --service-cidr=10.96.0.0/16 \
    --pod-network-cidr=172.20.0.0/16 \
    --cri-socket unix:///var/run/containerd/containerd.sock


# k8s查看安装日志
Your Kubernetes control-plane has initialized successfully!
To start using your cluster, you need to run the following as a regular user:

  mkdir -p $HOME/.kube
  sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config
  sudo chown $(id -u):$(id -g) $HOME/.kube/config

Alternatively, if you are the root user, you can run:

  export KUBECONFIG=/etc/kubernetes/admin.conf

You should now deploy a pod network to the cluster.
Run "kubectl apply -f [podnetwork].yaml" with one of the options listed at:
  https://kubernetes.io/docs/concepts/cluster-administration/addons/

You can now join any number of control-plane nodes by copying certificate authorities
and service account keys on each node and then running the following as root:

  kubeadm join ubuntu:6443 --token ndr3bo.llkmrdlkm8zvijuu \
        --discovery-token-ca-cert-hash sha256:691f6c2d9acf855948fbe2f9c3d095d880371163b5ed30142babbfd56bb03a25 \
        --control-plane 

Then you can join any number of worker nodes by running the following on each as root:

kubeadm join ubuntu:6443 --token ndr3bo.llkmrdlkm8zvijuu \
        --discovery-token-ca-cert-hash sha256:691f6c2d9acf855948fbe2f9c3d095d880371163b5ed30142babbfd56bb03a25


# 安装网络插件和监控
kubectl apply -f /etc/kubernetes/calico/calico.yaml
kubectl apply -f /etc/kubernetes/metrics-server/high-availability-1.21+.yaml 


# 去除主节点不支持调度污点
kubectl taint nodes [node-name] node-role.kubernetes.io/control-plane-

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