import tarfile
import os
import argparse


def create_tarball_from_current_directory(output_filename=None, include_arch=None):
    if output_filename is None:
        if include_arch:
            output_filename = f"Xdeploy-{include_arch}.tar.gz"
        else:
            output_filename = "Xdeploy-all.tar.gz"

    with tarfile.open(output_filename, "w:gz") as tar:
        for root, dirs, files in os.walk(os.getcwd()):
            parts = root.split(os.sep)
            if "repo" in parts:
                repo_index = parts.index("repo")
                # 检查当前目录是否是repo下的项目子目录（即repo/<project>）
                if len(parts) == repo_index + 2:
                    # 当前目录是repo/<project>，处理其子目录
                    if include_arch:
                        if include_arch in dirs:
                            dirs[:] = [include_arch]
                        else:
                            dirs[:] = []
                    else:
                        pass

                    files[:] = []

            for file in files:
                # 排除生成的tar包文件
                if file == output_filename:
                    continue

                file_path = os.path.join(root, file)
                arcname = os.path.relpath(file_path, os.getcwd())
                arcname = os.path.join("Xdeploy", arcname)
                tar.add(file_path, arcname=arcname)


def main():
    # 创建命令行参数解析器
    parser = argparse.ArgumentParser(
        description="Create tarball from current directory."
    )
    parser.add_argument(
        "arch",
        nargs="?",
        default=None,
        help="The architecture to include (e.g., x86_64). If not provided, all architectures will be included.",
    )

    args = parser.parse_args()

    create_tarball_from_current_directory(include_arch=args.arch)


if __name__ == "__main__":
    main()

# python tar.py x86_64
