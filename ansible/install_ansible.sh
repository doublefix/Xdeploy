#!/bin/bash

# sudo apt install -y python3 python3-pip python3-venv

echo "Creating virtual environment..."
python3 -m venv ansible-env

echo "Activating virtual environment..."
source ansible-env/bin/activate

echo "Installing ansible_core-2.17.2..."
pip install ansible/releases/ansible_core-2.17.2-py3-none-any.whl

echo "Verifying Ansible Core installation..."
ansible --version

echo "Deactivating virtual environment..."
deactivate

echo "Install ansible complete."