```bash
# 更新 apt包索引并安装使用 Kubernetes apt仓库所需要的包：
sudo apt-get update
# apt-transport-https may be a dummy package; if so, you can skip that package
sudo apt-get install -y apt-transport-https ca-certificates curl gpg

# 下载 Google Cloud 公开签名秘钥：
# If the directory `/etc/apt/keyrings` does not exist, it should be created before the curl command, read the note below.# sudo mkdir -p -m 755 /etc/apt/keyrings
curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.29/deb/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg


# 添加 Kubernetes apt仓库：
# This overwrites any existing configuration in /etc/apt/sources.list.d/kubernetes.list
echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.29/deb/ /' | sudo tee /etc/apt/sources.list.d/kubernetes.list

# 下载离线版本
apt-get download kubelet kubeadm kubectl
# 安装离线版本
sudo dpkg -i ~/k8s-packages/*.deb

# 更新 apt 包索引，安装 kubelet、kubeadm 和 kubectl，并锁定其版本：
sudo apt-get update
sudo apt-get install -y kubelet kubeadm kubectl

# 锁定与解锁升级
sudo apt-mark hold kubelet kubeadm kubectl
sudo apt-mark unhold kubelet kubeadm kubectl
sudo apt-get install -y kubelet=1.29.6-1.1 kubeadm=1.29.6-1.1 kubectl=1.29.6-1.1

# 查看版本列表
apt-cache madison kubelet kubeadm kubectl
sudo apt-get install -y kubelet=1.29.2-00 kubeadm=1.29.2-00 kubectl=1.29.2-00

# 查看安装版本
kubelet version
```
