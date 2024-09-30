## Install

Because depend systemd, so it note so easy.

```bash
git clone https://github.com/jupyterhub/the-littlest-jupyterhub.git
docker build -t tljh-systemd . -f integration-tests/Dockerfile
docker run \
  --privileged \
  --detach \
  --name=tljh-dev \
  --publish 12000:80 \
  --mount type=bind,source="$(pwd)",target=/srv/src \
  tljh-systemd
```

# Add admin
```bash
apt-get update
python3 /srv/src/bootstrap/bootstrap.py --admin admin:admin

# https://github.com/conda-forge/miniforge/releases/download/23.1.0-1/Mambaforge-23.1.0-1-Linux-x86_64.sh
```