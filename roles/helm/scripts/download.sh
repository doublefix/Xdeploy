#!/bin/bash

VERSION="v3.15.4"
OS="linux"
ARCH="amd64"

if [ ! -z "$1" ]; then
  VERSION=$1
fi

if [ ! -z "$2" ]; then
  OS=$2
fi

if [ ! -z "$3" ]; then
  ARCH=$3
fi

URL="https://get.helm.sh/helm-${VERSION}-${OS}-${ARCH}.tar.gz"

wget -P roles/helm/release $URL
