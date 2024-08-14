#!/bin/bash

KUBE_CONFIG="v0.16.2"
KUBE_CONF_DIR="roles/kubernetes/release/x86_64/conf"


mkdir -p "$KUBE_CONF_DIR"

echo "Downloading kubelet.service..."
wget -P "$KUBE_CONF_DIR" "https://raw.githubusercontent.com/kubernetes/release/${KUBE_CONFIG}/cmd/krel/templates/latest/kubelet/kubelet.service"
echo "kubelet.service downloaded successfully"

echo "Downloading 10-kubeadm.conf..."
wget -P "$KUBE_CONF_DIR" "https://raw.githubusercontent.com/kubernetes/release/${KUBE_CONFIG}/cmd/krel/templates/latest/kubeadm/10-kubeadm.conf"
echo "10-kubeadm.conf downloaded successfully"
