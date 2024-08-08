#!/bin/bash

# 单独配置 cgroup 控制器委派，下放到普通用户
systemd_conf_dir="/etc/systemd/system/user@.service.d"
delegate_conf_file="$systemd_conf_dir/delegate.conf"

if [ -d "$systemd_conf_dir" ]; then
    echo "$systemd_conf_dir 目录已存在。"
else
    echo "创建 $systemd_conf_dir 目录..."
    sudo mkdir -p "$systemd_conf_dir"
fi

if [ -f "$delegate_conf_file" ]; then
    echo "$delegate_conf_file 文件已存在, 请手动检查文件内容。"
else
    echo "创建 $delegate_conf_file 文件..."
    echo -e "[Service]\nDelegate=cpu cpuset io memory pids" | sudo tee "$delegate_conf_file"
    sudo systemctl daemon-reload
fi

echo "请重新登录或重启主机以使更改生效。"