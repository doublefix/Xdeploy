#!/bin/bash

ARCH="amd64"

CNI_PLUGINS_VERSION="v1.5.1"
CNI_DEST_DIR="roles/kubernetes/release/x86_64/cni"

CRICTL_VERSION="v1.31.0"
CRI_DEST_DIR="roles/kubernetes/release/x86_64/cri"

KUBE_VERSION="v1.31.0"
KUBE_CONFIG="v0.16.2"
KUBE_DEST_DIR="roles/kubernetes/release/x86_64/main"
KUBE_CONF_DIR="roles/kubernetes/release/x86_64/conf"

mkdir -p "$CNI_DEST_DIR"
mkdir -p "$CRI_DEST_DIR"
mkdir -p "$KUBE_DEST_DIR"
mkdir -p "$KUBE_CONF_DIR"

# 下载 CNI 插件
echo "Downloading CNI plugins version ${CNI_PLUGINS_VERSION}..."
wget -P "$CNI_DEST_DIR" "https://github.com/containernetworking/plugins/releases/download/${CNI_PLUGINS_VERSION}/cni-plugins-linux-${ARCH}-${CNI_PLUGINS_VERSION}.tgz"
echo "CNI plugins downloaded successfully"

# 下载 CRI 工具
echo "Downloading CRI tools version ${CRICTL_VERSION}..."
wget -P "$CRI_DEST_DIR" "https://github.com/kubernetes-sigs/cri-tools/releases/download/${CRICTL_VERSION}/crictl-${CRICTL_VERSION}-linux-${ARCH}.tar.gz"
echo "CRI tools downloaded successfully"

# 下载 kubeadm, kubelet, kubectl
echo "Downloading kubeadm version ${KUBE_VERSION}..."
wget -P "$KUBE_DEST_DIR" "https://dl.k8s.io/release/${KUBE_VERSION}/bin/linux/${ARCH}/kubeadm"
echo "kubeadm downloaded successfully"

echo "Downloading kubelet version ${KUBE_VERSION}..."
wget -P "$KUBE_DEST_DIR" "https://dl.k8s.io/release/${KUBE_VERSION}/bin/linux/${ARCH}/kubelet"
echo "kubelet downloaded successfully"

echo "Downloading kubectl version ${KUBE_VERSION}..."
wget -P "$KUBE_DEST_DIR" "https://dl.k8s.io/release/${KUBE_VERSION}/bin/linux/${ARCH}/kubectl"
echo "kubectl downloaded successfully"

# 下载 kubelet.service 和 10-kubeadm.conf
echo "Downloading kubelet.service..."
wget -P "$KUBE_CONF_DIR" "https://raw.githubusercontent.com/kubernetes/release/${KUBE_CONFIG}/cmd/krel/templates/latest/kubelet/kubelet.service"
echo "kubelet.service downloaded successfully"

echo "Downloading 10-kubeadm.conf..."
wget -P "$KUBE_CONF_DIR" "https://raw.githubusercontent.com/kubernetes/release/${KUBE_CONFIG}/cmd/krel/templates/latest/kubeadm/10-kubeadm.conf"
echo "10-kubeadm.conf downloaded successfully"
