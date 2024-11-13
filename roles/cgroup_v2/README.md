```bash
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
```

```bash
#!/bin/bash

# https://rootlesscontaine.rs/getting-started/common/cgroup2/

# Fedora (since 31)
# Arch Linux (since April 2021)
# openSUSE Tumbleweed (since c. 2021)
# Debian GNU/Linux (since 11)
# Ubuntu (since 21.10)
# RHEL and RHEL-like distributions (since 9)

# 检查是否启用了 cgroup v2
if [ -d /sys/fs/cgroup/cgroup.controllers ]; then
    echo "cgroup v2 已启用。"
    exit 0
fi

echo "cgroup v2 未启用。"

# 检查内核版本
kernel_version=$(uname -r | cut -d '.' -f 1-2)
required_version="4.15"

if [ "$(printf '%s\n' "$kernel_version" "$required_version" | sort -V | head -n1)" != "$required_version" ]; then
    echo "内核版本 $kernel_version 不支持 cgroup v2，需升级到 4.15 或更高版本。"
    exit 1
fi

# 检查 systemd 版本
systemd_version=$(systemctl --version | head -n 1 | awk '{print $2}')
required_systemd_version="244"

if [ "$(printf '%s\n' "$systemd_version" "$required_systemd_version" | sort -V | head -n1)" != "$required_systemd_version" ]; then
    echo "systemd 版本 $systemd_version 不支持 cgroup v2 委派，需升级到 244 或更高版本。"
    exit 1
fi

echo "启用 cgroup v2 和控制器委派..."

# 启用 cgroup v2
grub_file="/etc/default/grub"
if grep -q "systemd.unified_cgroup_hierarchy=1" "$grub_file"; then
    echo "GRUB 配置文件已包含 systemd.unified_cgroup_hierarchy=1。"
else
    echo "将 systemd.unified_cgroup_hierarchy=1 添加到 GRUB 配置文件中..."
    sudo sed -i 's/^\(GRUB_CMDLINE_LINUX_DEFAULT="[^"]*\)"/\1 systemd.unified_cgroup_hierarchy=1"/' "$grub_file"
    sudo update-grub
fi

# 配置 cgroup 控制器委派，为普通用户加上: cpu cpuset io memory pids
# cat /sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/cgroup.controllers
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

exit 0
```