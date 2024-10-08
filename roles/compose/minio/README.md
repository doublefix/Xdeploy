```bash
# https://min.io/docs/minio/container/index.html

# Docker
mkdir -p ${HOME}/minio/data
mkdir -p ${HOME}/minio/certs
# private.key
# public.crt

docker run -d \
   -p 9000:9000 \
   -p 9001:9001 \
   --name minio \
   -v ~/minio/data:/data \
   -v ~/minio/certs:/root/.minio/certs \
   -e "MINIO_ROOT_USER=ROOTNAME" \
   -e "MINIO_ROOT_PASSWORD=CHANGEME123" \
   --restart unless-stopped \
   quay.io/minio/minio:RELEASE.2024-10-02T17-50-41Z  server /data --console-address ":9001" --certs-dir /root/.minio/certs

docker rm minio

# Linux
wget https://dl.min.io/server/minio/release/linux-amd64/minio
chmod +x minio
sudo mv minio /usr/local/bin/
minio server ~/minio --console-address :9001
```