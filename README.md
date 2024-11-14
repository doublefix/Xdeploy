# Infrahub

The project is designed to help developers install essential software.
K8s version CHANGELOG

## How to use ansible

```bash
ssh-keygen -t rsa -b 2048
ssh-copy-id username@hostname
```

## Install kube

Execute in sequence

```bash
ansible-playbook playbooks/linux_kernel_opt.yml
ansible-playbook playbooks/cgroup_v2.yml

ansible-playbook playbooks/nerdctl.yml -e "arch=x86_64 version=1.7.6"

ansible-playbook playbooks/conntrack.yml -e "arch=x86_64 version=1.4.7"
ansible-playbook playbooks/socat.yml -e "arch=x86_64 version=1.7.4"
ansible-playbook playbooks/cri.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/cni.yml -e "arch=x86_64 version=v1.5.1"
ansible-playbook playbooks/kubelet.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/kubeadm.yml -e "arch=x86_64 version=v1.31.0"

# 初始化控制节点,只需要主节点执行,详细查看roles/kubeadm/README.md,尽量手动执行

ansible-playbook playbooks/calico.yml
ansible-playbook playbooks/metrics_server.yml

ansible-playbook playbooks/kubectl.yml -e "arch=x86_64 version=v1.31.0"
ansible-playbook playbooks/helm.yml -e "arch=x86_64 version=v3.15.4"
ansible-playbook playbooks/docker.yml -e "arch=x86_64 version=27.1.2"
ansible-playbook playbooks/docker_compose.yml -e "arch=x86_64 version=v2.29.2"
ansible-playbook playbooks/docker_buildx.yml -e "arch=x86_64 version=v0.16.2"
```

## Clean kube

Execute in sequence

```bash
# Delete node
kubectl cordon [node-name]
kubectl drain [node-name] --delete-emptydir-data --ignore-daemonsets --force
kubectl delete node [node-name]
# Reset node
sudo kubeadm reset
# Init network
sudo iptables -F
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo iptables -X
# Remove container
sudo crictl ps -a
sudo crictl rm $(sudo crictl ps -a -q)
# Remove image
sudo crictl images
sudo crictl rmi $(sudo crictl images -q)

ansible-playbook playbooks/stop/kubelet.yml
ansible-playbook playbooks/clean/kube_node_pod_image.yml
ansible-playbook playbooks/clean/kubeadm.yml
ansible-playbook playbooks/clean/kubelet.yml
ansible-playbook playbooks/clean/nerdctl.yml

ansible-playbook playbooks/clean/kubectl.yml
ansible-playbook playbooks/clean/helm.yml
ansible-playbook playbooks/clean/docker.yml
ansible-playbook playbooks/clean/docker_compose.yml
ansible-playbook playbooks/clean/docker_buildx.yml

```
