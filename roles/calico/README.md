## 版本与环境要求

k8s 版本与 calico 镜像需要对应，需要 cni 如/etc/cni/net.d 和/opt/cni/bin，/etc/cni/net.d 需要在 containerd 的配置，/opt/cni/bin 需要单独下载，详见 CNI
https://docs.tigera.io/calico/latest/getting-started/kubernetes/requirements

## 准备 yaml 配置

```bash
# 下载插件yaml
https://github.com/projectcalico/calico

curl https://raw.githubusercontent.com/projectcalico/calico/v3.27.2/manifests/calico.yaml -O
curl https://raw.githubusercontent.com/projectcalico/calico/v3.28.1/manifests/calico.yaml -O

curl -o roles/calico/conf/v3.29.2/calico.yaml https://raw.githubusercontent.com/projectcalico/calico/v3.29.2/manifests/calico.yaml


```

---

Supported versions
We test Calico v3.29 against the following Kubernetes versions. Other versions may work, but we are not actively testing them.

v1.29
v1.30
v1.31

## Error

calico 自动选择网卡不正确

```bash
# 网卡使用错误
# calico/node is not ready: BIRD is not ready: BGP not established with
https://blog.csdn.net/m0_53563073/article/details/135114128
https://blog.csdn.net/qq_33745102/article/details/126968473
https://docs.tigera.io/calico/latest/networking/ipam/ip-autodetection#change-the-autodetection-method

```

解决方案

```bash
# 检查calico pod内部选择的ip
cat /etc/calico/confd/config/bird.cfg|grep router
# 在节点查询网卡
ip a|grep xxx

# 示例：使用bond0网卡100.80.165.55是错误的，应该是使用tailscale0网卡10.32.0.145
48: bond0: <BROADCAST,MULTICAST,MASTER,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether 26:87:3e:a0:82:f8 brd ff:ff:ff:ff:ff:ff
    inet 10.32.0.145/24 brd 10.32.0.255 scope global bond0
       valid_lft forever preferred_lft forever
    inet6 fe80::2487:3eff:fea0:82f8/64 scope link
       valid_lft forever preferred_lft forever
56: tailscale0: <POINTOPOINT,MULTICAST,NOARP,UP,LOWER_UP> mtu 1280 qdisc fq_codel state UNKNOWN group default qlen 500
    link/none
    inet 100.80.165.55/32 scope global tailscale0
       valid_lft forever preferred_lft forever

# 几种解决方法
# 匹配正确的网卡bond0
kubectl set env daemonset/calico-node -n kube-system IP_AUTODETECTION_METHOD=interface=bond0
# 排除tailscale0
kubectl set env daemonset/calico-node -n kube-system IP_AUTODETECTION_METHOD=interface=skip-interface=tailscale0
# 能够ping通目标的网卡
kubectl set env daemonset/calico-node -n kube-system IP_AUTODETECTION_METHOD=interface=can-reach=www.google.com


```
