## 版本与环境要求

k8s 版本与 calico 镜像需要对应，需要 cni 如/etc/cni/net.d 和/opt/cni/bin，/etc/cni/net.d 需要在 containerd 的配置，/opt/cni/bin 需要单独下载，详见 CNI
https://docs.tigera.io/calico/latest/getting-started/kubernetes/requirements

## 准备 yaml 配置

```bash
# 下载插件yaml
https://github.com/projectcalico/calico

curl https://raw.githubusercontent.com/projectcalico/calico/v3.27.2/manifests/calico.yaml -O
curl https://raw.githubusercontent.com/projectcalico/calico/v3.28.1/manifests/calico.yaml -O

curl -o roles/calico/conf/v3.29.2/calico.yaml https://raw.githubusercontent.com/projectcalico/calico/v3.29.2/manifests/calico.yaml


```

---

Supported versions
We test Calico v3.29 against the following Kubernetes versions. Other versions may work, but we are not actively testing them.

v1.29
v1.30
v1.31
