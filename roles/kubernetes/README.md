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