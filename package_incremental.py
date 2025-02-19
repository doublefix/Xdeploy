import os
import tarfile
import pathspec
import yaml
from datetime import datetime


def create_code_tar_from_gitignore(
    gitignore_path, output_tar="xdeploy-incremental.tar.gz"
):
    try:
        with open(gitignore_path, "r") as f:
            spec = pathspec.GitIgnoreSpec.from_lines(f)
    except FileNotFoundError:
        print(f"Error: {gitignore_path} not found.")
        return
    except Exception as e:
        print(f"Error reading {gitignore_path}: {e}")
        return

    try:
        with tarfile.open(output_tar, "w:gz") as tar:
            for root, dirs, files in os.walk(".", topdown=True):
                dirs[:] = [
                    d
                    for d in dirs
                    if not spec.match_file(os.path.relpath(os.path.join(root, d), "."))
                ]

                if ".git" in dirs:
                    dirs.remove(".git")

                for file in files:
                    file_full_path = os.path.join(root, file)
                    rel_path = os.path.relpath(file_full_path, ".")

                    if file == os.path.basename(output_tar):
                        print(f"Ignored: {rel_path}")
                        continue

                    if not spec.match_file(rel_path):
                        print(f"Adding: {rel_path}")
                        tar.add(file_full_path, arcname=rel_path)
                    else:
                        print(f"Ignored: {rel_path}")

            for line in open(gitignore_path, "r"):
                if line.startswith("!"):
                    included_file = line[1:].strip()
                    if os.path.exists(included_file):
                        print(f"Adding explicitly included file: {included_file}")
                        tar.add(included_file, arcname=included_file)

            with open("repo/incremental_load.yaml", "r") as f:
                config = yaml.safe_load(f)
            if "binary" in config:
                for binary, platforms in config["binary"].items():
                    for platform, versions in platforms.items():
                        for version in versions:
                            # 拼接路径
                            binary_path = os.path.join(
                                "repo", binary, platform, version
                            )
                            if os.path.exists(binary_path):
                                print(f"Adding binary: {binary_path}")
                                tar.add(
                                    binary_path,
                                    arcname=os.path.relpath(binary_path, "."),
                                )
                            else:
                                print(f"Warning: Binary path not found: {binary_path}")

            # 处理 images 配置
            if "images" in config:
                for platform, images in config["images"].items():
                    for image in images:
                        image_name = image.replace("/", "-").replace(
                            ":", "_"
                        )  # 规范化 image 名称
                        image_path = os.path.join(
                            "repo", "images", platform, f"{image_name}.tar"
                        )
                        if os.path.exists(image_path):
                            print(f"Adding image: {image_path}")
                            tar.add(
                                image_path, arcname=os.path.relpath(image_path, ".")
                            )
                        else:
                            print(f"Warning: Image path not found: {image_path}")

    except Exception as e:
        print(f"Error creating tar file {output_tar}: {e}")


def process_incremental_load(config_path, output_tar="xdeploy-incremental.tar.gz"):
    try:
        # 读取 YAML 配置文件
        with open(config_path, "r") as f:
            config = yaml.safe_load(f)

        # 判断是否需要打包代码
        if config.get("code", False):
            create_code_tar_from_gitignore(".gitignore", output_tar)

    except FileNotFoundError:
        print(f"Error: {config_path} not found.")
    except yaml.YAMLError as e:
        print(f"Error reading YAML file: {e}")
    except Exception as e:
        print(f"Error processing {config_path}: {e}")


if __name__ == "__main__":
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"xdeploy-incremental_{timestamp}.tar.gz"
    process_incremental_load("repo/incremental_load.yaml", filename)
