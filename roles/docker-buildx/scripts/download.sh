#!/bin/bash

VERSION="v0.16.2"
ARCH="linux-amd64"

if [ ! -z "$1" ]; then
  VERSION=$1
fi

if [ ! -z "$2" ]; then
  ARCH=$2
fi

mkdir -p roles/docker-buildx/release/

wget -O roles/docker-buildx/release/buildx-${VERSION}.${ARCH} https://github.com/docker/buildx/releases/download/${VERSION}/buildx-${VERSION}.${ARCH}
