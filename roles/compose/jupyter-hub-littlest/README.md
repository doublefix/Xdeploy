## Install

Because depend systemd, so it note so easy. Offical website: https://tljh.jupyter.org/en/latest/contributing/dev-setup.html

```bash
git clone https://github.com/jupyterhub/the-littlest-jupyterhub.git
docker build -t tljh-systemd . -f integration-tests/Dockerfile
docker run \
  --privileged \
  --detach \
  --name=tljh-dev \
  --publish 12000:80 \
  --mount type=bind,source="$(pwd)",target=/srv/src \
  --restart=always \
  tljh-systemd
```

## Add admin

```bash
apt-get update
python3 /srv/src/bootstrap/bootstrap.py --admin admin:passwd

# https://github.com/conda-forge/miniforge/releases/download/23.1.0-1/Mambaforge-23.1.0-1-Linux-x86_64.sh
```

## Script

```bash
python3 /srv/src/bootstrap/bootstrap.py：

# Install
python3 -m tljh.installer：

# Reload, tljh/jupyterhub_config.py、tljh/configurer.py、/opt/tljh/config/
tljh-config reload hub：
```
