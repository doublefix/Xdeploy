#!/bin/bash

# 获取 k8s.io 命名空间中的所有镜像
images=$(sudo ctr -n k8s.io images ls -q)

# 检查是否有镜像
if [ -z "$images" ]; then
  echo "No images found in k8s.io namespace."
else
  # 删除所有镜像
  echo "Deleting the following images:"
  echo "$images"
  sudo ctr -n k8s.io images delete $images
  echo "All images deleted."
fi