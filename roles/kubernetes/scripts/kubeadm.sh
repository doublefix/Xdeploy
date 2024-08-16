sudo kubeadm init \
    --apiserver-advertise-address=192.168.1.1 \
    --control-plane-endpoint=master \
    --kubernetes-version v1.29.6 \
    --service-cidr=10.96.0.0/16 \
    --pod-network-cidr=172.20.0.0/16 \
    --cri-socket unix:///var/run/containerd/containerd.sock
