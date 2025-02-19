import os
import tarfile
import pathspec

def create_tar_from_gitignore(gitignore_path, output_tar='code.tar.gz'):
    # 读取.gitignore文件并解析规则
    try:
        with open(gitignore_path, 'r') as f:
            spec = pathspec.GitIgnoreSpec.from_lines(f)
    except FileNotFoundError:
        print(f"Error: {gitignore_path} not found.")
        return
    except Exception as e:
        print(f"Error reading {gitignore_path}: {e}")
        return
    
    # 创建tar.gz文件
    try:
        with tarfile.open(output_tar, 'w:gz') as tar:
            for root, dirs, files in os.walk('.', topdown=True):
                # 修改遍历的目录列表，提前排除被忽略的目录
                dirs[:] = [d for d in dirs if not spec.match_file(os.path.relpath(os.path.join(root, d), '.'))]
                
                for file in files:
                    file_full_path = os.path.join(root, file)
                    rel_path = os.path.relpath(file_full_path, '.')

                    # 检查文件是否未被.gitignore规则忽略
                    if not spec.match_file(rel_path):
                        print(f"Adding: {rel_path}")
                        # 将文件添加到tar包中，保持相对路径
                        tar.add(file_full_path, arcname=rel_path)
                    else:
                        print(f"Ignored: {rel_path}")
    except Exception as e:
        print(f"Error creating tar file {output_tar}: {e}")

if __name__ == '__main__':
    create_tar_from_gitignore('.gitignore')