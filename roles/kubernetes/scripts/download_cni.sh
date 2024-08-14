#!/bin/bash

ARCH="amd64"

CNI_PLUGINS_VERSION="v1.5.1"
CNI_DEST_DIR="roles/kubernetes/release/x86_64/cni"

CRICTL_VERSION="v1.30.0"
CRI_DEST_DIR="roles/kubernetes/release/x86_64/cri"

mkdir -p "$CNI_DEST_DIR"
mkdir -p "$CRI_DEST_DIR"

echo "Downloading CNI plugins version ${CNI_PLUGINS_VERSION}..."
wget -P "$CNI_DEST_DIR" "https://github.com/containernetworking/plugins/releases/download/${CNI_PLUGINS_VERSION}/cni-plugins-linux-${ARCH}-${CNI_PLUGINS_VERSION}.tgz"
echo "CNI plugins download successfully"

echo "Downloading CRI tools version ${CRICTL_VERSION}..."
wget -P "$CRI_DEST_DIR" "https://github.com/kubernetes-sigs/cri-tools/releases/download/${CRICTL_VERSION}/crictl-${CRICTL_VERSION}-linux-${ARCH}.tar.gz"
echo "CRI tools download and extract successfully"
