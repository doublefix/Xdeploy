## ENV

```bash
# Test
ansible all --list-hosts
ansible all -m ping

# Manually
pip download -d wheels ansible-core==2.17.2 ansible-runner==2.4.0 Flask==3.0.3
pip install wheels/*.whl

# Download Linux package source
https://packages.debian.org/
https://rpmfind.net/

# 创建一个集群的基础环境
# 1.保证dns数量不能太多2个，1个最佳，否则部分组件识别不准确

```

## Build Image

```bash
TAG=test HUB=docker.io make image

ssh-keygen -t rsa -b 2048
ssh-copy-id username@hostname
```

## 安装完成 k8s 后构建平台

手动拉取镜像，推送镜像

```bash
docker pull docker.io/calico/cni:v3.29.2
docker pull docker.io/calico/node:v3.29.2
docker pull docker.io/calico/kube-controllers:v3.29.2

docker save -o calico-cni-v3.29.2.tar docker.io/calico/cni:v3.29.2
docker save -o calico-node-v3.29.2.tar docker.io/calico/node:v3.29.2
docker save -o calico-kube-controllers-v3.29.2.tar docker.io/calico/kube-controllers:v3.29.2

nerdctl load -i calico-cni-v3.29.2.tar --namespace k8s.io
nerdctl load -i calico-node-v3.29.2.tar --namespace k8s.io
nerdctl load -i calico-kube-controllers-v3.29.2.tar --namespace k8s.io

```
