```bash
# 1.7.6
wget -P roles/containerd/release/assets/ https://github.com/containerd/nerdctl/releases/download/v1.7.6/nerdctl-full-1.7.6-linux-amd64.tar.gz

# Install
ansible-playbook playbooks/nerdctl.yml
ansible-playbook playbooks/nerdctl.yml -e "operation=install"

# Uninstall
ansible-playbook playbooks/nerdctl.yml -e "operation=uninstall"
```
