# Xdeploy

The project is designed to help developers install essential software.

## Setup

```bash
# Env
sudo apt install python3.12 python3.12-dev
ls /usr/lib/x86_64-linux-gnu/libpython3.1*
python3 -m venv venv --system-site-packages
source venv/bin/activate
pip install -r requirements.txt
ansible-galaxy collection install ansible.posix
ansible-galaxy collection install community.general
deactivate
# Package
python tar.py x86_64
# Get Images
python load_images.py
```

## Install kube

Execute in sequence

```bash
ansible-playbook playbooks/cmd.yml -e '{"cmd": ["echo", "Hello", "World"]}' -v

ansible-playbook playbooks/linux_kernel_opt.yml
ansible-playbook playbooks/cgroup_v2.yml

ansible-playbook playbooks/nerdctl.yml -e "arch=x86_64 version=v1.7.6"

ansible-playbook playbooks/conntrack.yml -e "arch=x86_64 version=1.4.7"
ansible-playbook playbooks/socat.yml -e "arch=x86_64 version=1.7.4"
ansible-playbook playbooks/cri.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/cni.yml -e "arch=x86_64 version=v1.5.1"
ansible-playbook playbooks/kubelet.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/kubeadm.yml -e "arch=x86_64 version=v1.31.0"

ansible-playbook playbooks/calico.yml
ansible-playbook playbooks/metrics_server.yml
# Load images
ansible-playbook playbooks/images.yml
# Install nvidia-container-toolkit
ansible-playbook playbooks/nvidia-container-toolkit.yaml -e "arch=x86_64 version=1.17.4"

ansible-playbook playbooks/kubectl.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/helm.yml -e "arch=x86_64 version=v3.15.4"
ansible-playbook playbooks/docker.yml -e "arch=x86_64 version=27.1.2"
ansible-playbook playbooks/docker_compose.yml -e "arch=x86_64 version=v2.29.2"
ansible-playbook playbooks/docker_buildx.yml -e "arch=x86_64 version=v0.16.2"

python load_images.py
python package_incremental.py
python update.py backupcode
python update.py load xdeploy-incremental_20250219_195822.tar.gz
```

## Clean kube

Execute in sequence

```bash
# Delete node
kubectl cordon [node-name]
kubectl drain [node-name] --ignore-daemonsets --delete-local-data
kubectl delete node [node-name]
# Reset node
sudo kubeadm reset
# Init network
sudo iptables -F
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo iptables -X

ansible-playbook playbooks/stop/kubelet.yml

ansible-playbook playbooks/clean/pod_image.yml
ansible-playbook playbooks/clean/nerdctl.yml -e "version=v1.7.6"
ansible-playbook playbooks/clean/kubelet.yml
ansible-playbook playbooks/clean/kubeadm.yml

ansible-playbook playbooks/clean/kubectl.yml
ansible-playbook playbooks/clean/helm.yml
ansible-playbook playbooks/clean/docker.yml
ansible-playbook playbooks/clean/docker_compose.yml
ansible-playbook playbooks/clean/docker_buildx.yml

```

# 构建 k8s 平台

见 docs/op.md，以下三种版本必须相同

- CRI
- kubelet
- kubeadm
- kubectl

## 注意事项

1. 一个集群的节点解析，主节点 hosts 连所有从节点，每个从节点连接主节点

- 收集 hostname(大小写注意)+ip 制成 hosts
- 修改 hosts
- 修改 authorized_keys,把部署机的 key 放到被管理机器

2. 使用 Xdeploy
3. 部署完成后检查 INTERNAL-IP 是否符合预期
4. 下载预备安装文件有两种：下载二进制包，下载镜像

## 初始化集群

```bash
kubeadm init \
    --apiserver-advertise-address=NODEIP \
    --control-plane-endpoint=NODENAME \
    --kubernetes-version=v1.31.0 \
    --service-cidr=10.96.0.0/16 \
    --pod-network-cidr=172.20.0.0/16 \
    --cri-socket=unix:///var/run/containerd/containerd.sock \
    --image-repository=registry.aliyuncs.com/google_containers

```

INTERNAL-IP 不符合预期，手动指定

```bash
# check INTERNAL-IP
mkdir -p /etc/sysconfig
vim /etc/sysconfig/kubelet
KUBELET_EXTRA_ARGS="--node-ip=192.168.2.4"

sudo systemctl daemon-reload
sudo systemctl restart kubelet

```

## 复制容器内的二进制安装文件

```bash

nerdctl pull harbor.openpaper.co/chess/kubernetes:v1.31.0

IMAGE_ID=$(nerdctl image inspect --format '{{.ID}}' harbor.openpaper.co/chess/kubernetes:v1.31.0 | cut -d ':' -f 2)

mkdir -p /var/tmp/chess/"$IMAGE_ID"

nerdctl run --rm \
  -v /var/tmp/chess/"$IMAGE_ID":/extract \
  harbor.openpaper.co/chess/kubernetes:v1.31.0 \
  sh -c 'cp -r /archive/ /extract/'

ls -l /var/tmp/chess/"$IMAGE_ID"/

```
