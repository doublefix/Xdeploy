#!/bin/bash

os="linux"
arch="x86_64"
version="27.1.2"

while getopts o:a:v: flag
do
    case "${flag}" in
        o) os=${OPTARG};;
        a) arch=${OPTARG};;
        v) version=${OPTARG};;
    esac
done

wget -P roles/docker/release https://download.docker.com/${os}/static/stable/${arch}/docker-${version}.tgz
wget -P roles/docker/release https://download.docker.com/${os}/static/stable/${arch}/docker-rootless-extras-${version}.tgz

# roles/docker/scripts/download.sh -o linux -a arm64 -v 20.10.8
