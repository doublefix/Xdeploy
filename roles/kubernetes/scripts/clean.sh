#!/bin/bash

ARCH="amd64"

CNI_PLUGINS_VERSION="v1.5.1"
CNI_DEST_DIR="roles/kubernetes/release/x86_64/cni"

CRICTL_VERSION="v1.31.0"
CRI_DEST_DIR="roles/kubernetes/release/x86_64/cri"

KUBE_VERSION="v1.31.0"
KUBE_CONFIG="v0.16.2"
KUBE_DEST_DIR="roles/kubernetes/release/x86_64/main"

rm -f roles/kubernetes/release/x86_64/cni/*
rm -f roles/kubernetes/release/x86_64/cri/*
rm -f roles/kubernetes/release/x86_64/main/*

# # 删除 CNI 插件
# echo "Removing CNI plugins..."
# rm -f "$CNI_DEST_DIR/cni-plugins-linux-${ARCH}-${CNI_PLUGINS_VERSION}.tgz"
# echo "CNI plugins removed successfully"

# # 删除 CRI 工具
# echo "Removing CRI tools..."
# rm -f "$CRI_DEST_DIR/crictl-${CRICTL_VERSION}-linux-${ARCH}.tar.gz"
# echo "CRI tools removed successfully"

# # 删除 kubeadm, kubelet, kubectl
# echo "Removing kubeadm..."
# rm -f "$KUBE_DEST_DIR/kubeadm"
# echo "kubeadm removed successfully"

# echo "Removing kubelet..."
# rm -f "$KUBE_DEST_DIR/kubelet"
# echo "kubelet removed successfully"

# echo "Removing kubectl..."
# rm -f "$KUBE_DEST_DIR/kubectl"
# echo "kubectl removed successfully"
