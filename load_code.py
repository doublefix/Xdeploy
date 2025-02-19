import os
import tarfile

def parse_gitignore(gitignore_file):
    ignore_patterns = []
    include_patterns = []
    
    with open(gitignore_file, 'r') as f:
        lines = f.readlines()
        for line in lines:
            line = line.strip()
            if line.startswith('!'):
                include_patterns.append(line[1:])
            elif line and not line.startswith('#'):
                ignore_patterns.append(line)
    
    return ignore_patterns, include_patterns

def should_include(file_path, ignore_patterns, include_patterns):
    # Check if the file is explicitly included
    for include in include_patterns:
        if os.path.normpath(file_path).endswith(include):
            return True
    
    # Check if the file matches any ignore pattern
    for ignore in ignore_patterns:
        if os.path.normpath(file_path).endswith(ignore):
            return False
    
    return True

def create_tarball(source_dir, tarball_name, ignore_patterns, include_patterns):
    with tarfile.open(tarball_name, "w:gz") as tar:
        for dirpath, dirnames, filenames in os.walk(source_dir):
            # Check the directory to include or ignore
            if not should_include(dirpath, ignore_patterns, include_patterns):
                continue
            
            # Add the directory itself
            tar.add(dirpath, arcname=os.path.relpath(dirpath, source_dir))

            # Add the files in the directory
            for filename in filenames:
                file_path = os.path.join(dirpath, filename)
                if should_include(file_path, ignore_patterns, include_patterns):
                    tar.add(file_path, arcname=os.path.relpath(file_path, source_dir))

if __name__ == '__main__':
    source_dir = '.'  # Change this to the root directory of your code
    gitignore_file = '.gitignore'
    tarball_name = 'code.tar.gz'
    
    # Parse the .gitignore file
    ignore_patterns, include_patterns = parse_gitignore(gitignore_file)
    
    # Create the tarball
    create_tarball(source_dir, tarball_name, ignore_patterns, include_patterns)
    print(f'Tarball {tarball_name} created successfully!')
