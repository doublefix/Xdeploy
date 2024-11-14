import os
import urllib
import yaml


def load_yaml(file_path):
    with open(file_path, "r") as file:
        return yaml.safe_load(file)


def download_file(url, dest_path, overwrite=False):
    if os.path.exists(dest_path) and not overwrite:
        print(f"文件 {dest_path} 已存在，跳过下载。")
        return

    print(f"从 {url} 下载到 {dest_path}...")
    urllib.request.urlretrieve(url, dest_path)
    print("下载完成。")


def delete_file(dest_path):
    if os.path.exists(dest_path):
        os.remove(dest_path)
        print(f"已删除文件 {dest_path}")
    else:
        print(f"文件 {dest_path} 不存在，跳过删除。")
