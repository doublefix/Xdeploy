import os
import sys
import tarfile
import pathspec
import shutil
from datetime import datetime


def create_tar_from_gitignore(gitignore_path, output_tar_prefix="xdeploy-backup"):
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_tar = f"{output_tar_prefix}_{timestamp}.tar.gz"

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

                    # Skip files starting with 'xdeploy-incremental_'
                    if file.startswith("xdeploy-incremental_"):
                        print(f"Ignored: {rel_path} (matches xdeploy-incremental_)")
                        continue

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

        for root, dirs, files in os.walk(".", topdown=False):
            for file in files:
                if file.startswith("xdeploy-incremental_"):
                    print(f"Ignored: {rel_path} (matches xdeploy-incremental_)")
                    continue
                if file.startswith("update.py"):
                    print(f"Ignored: {rel_path} (matches xdeploy-incremental_)")
                    continue
                if file.startswith("backup"):
                    print(f"Ignored: {rel_path} (matches xdeploy-incremental_)")
                    continue
                file_full_path = os.path.join(root, file)
                rel_path = os.path.relpath(file_full_path, ".")

                if not spec.match_file(rel_path) and file != os.path.basename(
                    output_tar
                ):
                    print(f"Deleting: {rel_path}")
                    os.remove(file_full_path)

            if not os.listdir(root):
                print(f"Deleting empty directory: {root}")
                os.rmdir(root)

        backup_dir = os.path.join(os.getcwd(), "backup")
        if not os.path.exists(backup_dir):
            os.makedirs(backup_dir)

        date_folder = os.path.join(backup_dir, timestamp)
        os.makedirs(date_folder)

        destination = os.path.join(date_folder, output_tar)
        shutil.move(output_tar, destination)
        print(f"Moved {output_tar} to {destination}")

    except Exception as e:
        print(f"Error creating tar file {output_tar}: {e}")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        action = sys.argv[1]

        if action == "backupcode":
            # Backup code
            create_tar_from_gitignore(".gitignore")
            print("Backup completed.")

        elif action == "load":
            # Load and extract tar file
            if len(sys.argv) > 2:
                tar_file = sys.argv[2]
                if tar_file.endswith('.tar.gz') and os.path.exists(tar_file):
                    print(f"Extracting {tar_file}...")
                    with tarfile.open(tar_file, 'r:gz') as tar:
                        tar.extractall()
                    print(f"Extraction of {tar_file} complete.")
                else:
                    print(f"Error: {tar_file} is not a valid .tar.gz file or does not exist.")
            else:
                print("Error: Please provide a tar file to extract.")
        
        else:
            print("Error: Invalid argument. Use 'backupcode' to create a backup or 'load' to extract a tar file.")
    
    else:
        print("Error: No argument provided. Use 'backupcode' to create a backup or 'load' to extract a tar file.")