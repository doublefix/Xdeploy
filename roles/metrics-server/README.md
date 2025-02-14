```bash
https://github.com/kubernetes-sigs/metrics-server/
wget https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
# 修改metrics-server的yaml,添加tls
spec:
  template:
    spec:
      containers:
      - args:
        - --kubelet-insecure-tls
```
