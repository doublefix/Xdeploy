import subprocess
import os
import yaml

with open("repo/images.yml", "r") as file:
    images = yaml.safe_load(file)

current_directory = os.getcwd()

for architecture, image_list in images.items():
    if architecture == "x86_64":
        platform = "linux/amd64"
    elif architecture == "arrach64":
        platform = "linux/arm64"
    else:
        platform = None

    if platform:
        architecture_dir = os.path.join(current_directory, "repo", "images", architecture)
        os.makedirs(architecture_dir, exist_ok=True)

        for image in image_list:
            tar_filename = image.replace("/", "-").replace(":", "_") + ".tar"
            tar_path = os.path.join(architecture_dir, tar_filename)

            if os.path.exists(tar_path):
                print(f"镜像 {image} 的保存文件 {tar_path} 已存在，跳过保存。")
                continue

            print(f"正在拉取镜像: {image} ({platform})")
            try:
                subprocess.run(["docker", "pull", "--platform", platform, image], check=True)
                print(f"镜像 {image} 拉取完成！")
            except subprocess.CalledProcessError as e:
                print(f"拉取镜像 {image} 失败: {e}")
                continue

            print(f"正在保存镜像: {image} 到 {tar_path}")
            try:
                subprocess.run(["docker", "save", "-o", tar_path, image], check=True)
                print(f"镜像 {image} 保存为 {tar_path} 完成！")
            except subprocess.CalledProcessError as e:
                print(f"保存镜像 {image} 失败: {e}")

print("所有镜像处理完成！")
