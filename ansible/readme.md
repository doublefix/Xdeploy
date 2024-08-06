# Ansible
~~~bash
# 安装
bash ansible/install_ansible.sh

# 激活ansible环境
source ansible-env/bin/activate

# 退出ansible环境
deactivate
~~~

~~~bash
ansible all --list-hosts
ansible all -m ping
ansible-playbook playbook.yml
~~~