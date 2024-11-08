# Infrahub

The project is designed to help developers install essential software.
K8s version CHANGELOG

## How to use ansible

```bash
# 安装
bash ansible/install_ansible.sh
# 激活ansible环境
source ansible-env/bin/activate
# 安装依赖
pip install -r requirements.txt
# 退出ansible环境
deactivate
# 常用命令
ansible all --list-hosts
ansible all -m ping
ansible-playbook playbook.yml
# 使用示例：使用root安装
ansible-playbook playbooks/vim.yml --ask-become-pass
# 使用示例：动态获取服务器安装
ansible-playbook -i inventory/dynamic.py dev playbooks/site.yml

# 生成公钥
ssh-keygen -t rsa -b 2048
# 添加公钥到目标服务器，输入目标机器的密码
ssh-copy-id username@hostname
```

## Install kube

Execute in sequence

```bash
# 优化Linux内核等
ansible-playbook playbooks/linux_kernel_opt.yml
# 开启cgroup,更好的支持虚拟化
ansible-playbook playbooks/cgroup_v2.yml
# 安装nerdctl
ansible-playbook playbooks/nerdctl.yml
# 安装k8s
ansible-playbook playbooks/kubelet.yml
# 初始化控制节点,只需要主节点执行,详细查看roles/kubeadm/README.md,尽量手动执行
ansible-playbook playbooks/kubeadm.yml
# Node节点加入Cluster
ansible-playbook playbooks/kube_add_node.yml -e "control_plane=node:6443 kubeadm_token=xxxx.xxxxxxxxxxxx discovery_token_ca_cert_hash=sha256:xxxx is_control_plane=--control-plane"

# 安装kubectl
ansible-playbook playbooks/kubectl.yml
# 安装helm
ansible-playbook playbooks/helm.yml
# 安装docker
ansible-playbook playbooks/docker.yml
# 安装docker-compose
ansible-playbook playbooks/docker_compose.yml
# 安装docker-buildx
ansible-playbook playbooks/docker_buildx.yml
```

## Clean kube

Execute in sequence

```bash
# 驱逐节点,停止服务
kubectl cordon [node-name]
kubectl drain [node-name] --delete-emptydir-data --ignore-daemonsets --force
kubectl delete node [node-name]
# 初始化节点，删数据
sudo kubeadm reset
# 初始化网络
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

# 停止kubelet
ansible-playbook playbooks/stop/kubelet.yml
# 在移除的节点，删除pod和镜像
ansible-playbook playbooks/clean/kube_node_pod_image.yml
# 清理k8s
ansible-playbook playbooks/clean/kubelet.yml
# 清理nerdctl
ansible-playbook playbooks/clean/nerdctl.yml

# 清理kubectl
ansible-playbook playbooks/clean/kubectl.yml
# 清理helm
ansible-playbook playbooks/clean/helm.yml
# 清理docker
ansible-playbook playbooks/clean/docker.yml
# 清理docker-compose
ansible-playbook playbooks/clean/docker_compose.yml
# 清理docker-buildx
ansible-playbook playbooks/clean/docker_buildx.yml

```

```bash
# 使用接口启动任务
curl -X POST http://localhost:5000/run-playbook \
-H "Content-Type: application/json" \
-d '{
    "playbook_path": "playbooks/vim.yml",
    "inventory": {
        "servers": {
            "hosts": ["ubuntu-root"]
        }
    }
}'

curl -X POST http://localhost:5000/run-playbook \
-H "Content-Type: application/json" \
-d '{
    "playbook_path": "playbooks/vim.yml",
    "inventory": {
        "servers": {
            "hosts": ["ubuntu-root"]
        }
    },
    "extra_vars": {
        "variable1": "value1",
        "variable2": "value2"
    }
}'

# 查询任务运行结果
curl -X GET http://localhost:5000/task-status/1

# 装载二进制文件
curl -X POST http://localhost:5000/manage-tools \
    -H "Content-Type: application/json" \
    -d '{
        "tools": ["kubelet", "kubectl"],
        "archs": ["x86_64", "arrach64"],
        "mode": "download",
        "overwrite": false
    }'

```
