import os
import urllib.request
import yaml

TOOLS_TO_DOWNLOAD = ["kubeadm"]  #  ["kubectl"], ["kubeadm"], []

def load_yaml(file_path):
    with open(file_path, "r") as file:
        return yaml.safe_load(file)

def download_file(url, dest_path, overwrite=False):
    if os.path.exists(dest_path) and not overwrite:
        print(f"File {dest_path} already exists. Skipping download.")
        return

    print(f"Downloading from {url} to {dest_path}...")
    urllib.request.urlretrieve(url, dest_path)
    print("Download complete.")

def main(overwrite=False):
    yaml_data = load_yaml("meta.yml")

    for tool_name, arch_data in yaml_data['kubernetes'].items():
        if tool_name not in TOOLS_TO_DOWNLOAD:
            continue

        for arch, versions in arch_data.items():
            for version, files in versions.items():
                for file_info in files:
                    url = file_info['source']
                    name = file_info['name']
                    
                    dest_dir = f"roles/{tool_name}/release/{arch}/{version}"
                    os.makedirs(dest_dir, exist_ok=True)
                    
                    dest_path = os.path.join(dest_dir, name)
                    download_file(url, dest_path, overwrite=overwrite)

if __name__ == "__main__":
    main(overwrite=False)