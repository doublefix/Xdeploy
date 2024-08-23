#!/bin/bash

VERSION="v0.16.2"
ARCH="amd64"
OS="linux"

if [ ! -z "$1" ]; then
  VERSION=$1
fi

if [ ! -z "$2" ]; then
  ARCH=$2
fi

if [ ! -z "$3" ]; then
  OS=$3
fi

mkdir -p roles/docker-buildx/release/

wget -O roles/docker-buildx/release/buildx-${VERSION}.${OS}-${ARCH} https://github.com/docker/buildx/releases/download/${VERSION}/buildx-${VERSION}.${OS}-${ARCH}
