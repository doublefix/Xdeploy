```bash
# Download
bash roles/nerdctl/scripts/download_nerdctl.sh

# Install
ansible-playbook playbooks/nerdctl.yml
ansible-playbook playbooks/nerdctl.yml -e "operation=install"

# Uninstall
ansible-playbook playbooks/nerdctl.yml -e "operation=uninstall"
```
