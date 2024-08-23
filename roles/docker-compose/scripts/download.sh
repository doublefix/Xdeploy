#!/bin/bash

VERSION="v2.29.2"
ARCH="x86_64"
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

mkdir -p roles/docker-compose/release/${VERSION}/

wget -O roles/docker-compose/release/${VERSION}/docker-compose-${OS}-${ARCH} https://github.com/docker/compose/releases/download/${VERSION}/docker-compose-${OS}-${ARCH}
