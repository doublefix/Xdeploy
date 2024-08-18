# Infrahub

The project is designed to help developers install essential software.
K8s 最新版本号查看官方仓库的 CHANGELOG

## How to use ansible

```bash
# 安装
bash ansible/install_ansible.sh
# 激活ansible环境
source ansible-env/bin/activate
# 退出ansible环境
deactivate
# 常用命令
ansible all --list-hosts
ansible all -m ping
ansible-playbook playbook.yml
# 使用文件寻找服务器安装
ansible-playbook playbooks/vim.yml --ask-become-pass
# 动态获取服务器安装
ansible-playbook -i inventory/dynamic.py dev playbooks/site.yml
```

## Install

```bash
# 优化Linux内核等
ansible-playbook playbooks/linux_kernel_opt.yml
# 开启cgroup,更好的支持虚拟化
ansible-playbook playbooks/cgroup_v2.yml
# 安装nerdctl
ansible-playbook playbooks/nerdctl.yml
# 安装k8s
ansible-playbook playbooks/kubernetes.yml
# 安装kubectl
ansible-playbook playbooks/kubectl.yml
# 初始化控制节点,只需要主节点执行
ansible-playbook playbooks/kubeadm_init.yml

```

# Clean

```bash
# 停止kubectl
ansible-playbook playbooks/stop/kubectl.yml
# 清理kubectl
ansible-playbook playbooks/clean/kubectl.yml
# 清理k8s
ansible-playbook playbooks/clean/kubernetes.yml
# 清理nerdctl
ansible-playbook playbooks/clean/nerdctl.yml

```
