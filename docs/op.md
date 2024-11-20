## ENV

```bash
# Install dependency
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
deactivate

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
