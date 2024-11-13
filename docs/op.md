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

```
