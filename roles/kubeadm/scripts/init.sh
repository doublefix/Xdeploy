# 生产默认配置
kubeadm config print init-defaults > default-config.yaml
# 初始化集群
kubeadm init --config=/etc/kubernetes/init-control-plane.yml
