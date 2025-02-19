import os
import tarfile
import pathspec

def create_code_tar_from_gitignore(gitignore_path, output_tar='xdeploy-code.tar.gz'):
    try:
        with open(gitignore_path, 'r') as f:
            spec = pathspec.GitIgnoreSpec.from_lines(f)
    except FileNotFoundError:
        print(f"Error: {gitignore_path} not found.")
        return
    except Exception as e:
        print(f"Error reading {gitignore_path}: {e}")
        return
    
    try:
        with tarfile.open(output_tar, 'w:gz') as tar:
            for root, dirs, files in os.walk('.', topdown=True):
                dirs[:] = [d for d in dirs if not spec.match_file(os.path.relpath(os.path.join(root, d), '.'))]
                
                if '.git' in dirs:
                    dirs.remove('.git')
                
                for file in files:
                    file_full_path = os.path.join(root, file)
                    rel_path = os.path.relpath(file_full_path, '.')

                    if file == os.path.basename(output_tar):
                        print(f"Ignored: {rel_path}")
                        continue
                    
                    if not spec.match_file(rel_path):
                        print(f"Adding: {rel_path}")
                        tar.add(file_full_path, arcname=rel_path)
                    else:
                        print(f"Ignored: {rel_path}")
            
            for line in open(gitignore_path, 'r'):
                if line.startswith('!'):
                    included_file = line[1:].strip()
                    if os.path.exists(included_file):
                        print(f"Adding explicitly included file: {included_file}")
                        tar.add(included_file, arcname=included_file)
                        
    except Exception as e:
        print(f"Error creating tar file {output_tar}: {e}")

if __name__ == '__main__':
    create_code_tar_from_gitignore('.gitignore')
