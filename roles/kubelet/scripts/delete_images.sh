#!/bin/bash

# 获取正在运行的镜像列表
running_images=$(kubectl get pods --all-namespaces -o jsonpath="{..image}" | \
tr -s '[[:space:]]' '\n' | \
sort | \
uniq)

# 获取所有镜像列表
all_images=$(sudo ctr -n k8s.io images ls -q)

# 检查是否有镜像
if [ -z "$all_images" ]; then
  echo "No images found in k8s.io namespace."
else
  echo "Deleting images not in use:"
  # 遍历所有镜像
  for image in $all_images; do
    # 检查镜像是否在运行中
    if ! echo "$running_images" | grep -q "$image"; then
      echo "Deleting image: $image"
      sudo ctr -n k8s.io image delete $image
    fi
  done
  echo "All unused images deleted."
fi