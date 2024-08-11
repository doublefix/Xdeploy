```bash
# 1.7.6
wget -P roles/containerd/release/assets/ https://github.com/containerd/nerdctl/releases/download/v1.7.6/nerdctl-full-1.7.6-linux-amd64.tar.gz

roles/containerd/release/assets/
sudo tar Cxzvvf /usr/local nerdctl-full-1.7.6-linux-amd64.tar.gz

```

配置 cgroup 委派器

```bash
sudo roles/containerd/scripts/configure_cgroup_delegation.sh
```
