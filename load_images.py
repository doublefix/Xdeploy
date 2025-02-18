import subprocess
import os
import yaml

with open("repo/images.yml", "r") as file:
    images = yaml.safe_load(file)

current_directory = os.getcwd()

# 遍历每个平台及其镜像列表
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
            # 生成压缩包文件名
            tar_filename = image.replace("/", "-").replace(":", "_") + ".tar"
            tar_path = os.path.join(architecture_dir, tar_filename)

            # 如果文件已存在，则跳过该镜像
            if os.path.exists(tar_path):
                print(f"镜像 {image} 的保存文件 {tar_path} 已存在，跳过保存。")
                continue

            # 先执行 docker pull
            print(f"正在拉取镜像: {image} ({platform})")
            try:
                subprocess.run(["docker", "pull", "--platform", platform, image], check=True)
                print(f"镜像 {image} 拉取完成！")
            except subprocess.CalledProcessError as e:
                print(f"拉取镜像 {image} 失败: {e}")
                continue

            print(f"正在保存镜像: {image} 到 {tar_path}")
            # 执行 docker save 命令保存镜像
            try:
                subprocess.run(["docker", "save", "-o", tar_path, image], check=True)
                print(f"镜像 {image} 保存为 {tar_path} 完成！")
            except subprocess.CalledProcessError as e:
                print(f"保存镜像 {image} 失败: {e}")

print("所有镜像处理完成！")
