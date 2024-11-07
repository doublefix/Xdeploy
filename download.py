import os
import urllib.request
import yaml

def load_yaml(file_path):
    with open(file_path, "r") as file:
        return yaml.safe_load(file)

def download_kubectl(url, dest_path):
    print(f"Downloading kubectl from {url} to {dest_path}...")
    urllib.request.urlretrieve(url, dest_path)
    print("Download complete.")

def main():
    yaml_data = load_yaml("meta.yml")

    for arch, arch_data in yaml_data['kubernetes']['kubectl'].items():
        for version, files in arch_data.items():
            for file_info in files:
                url = file_info['source']
                name = file_info['name']
                
                dest_dir = f"roles/kubectl/release/{arch}/{version}"
                os.makedirs(dest_dir, exist_ok=True)
                
                dest_path = os.path.join(dest_dir, name)
                download_kubectl(url, dest_path)

if __name__ == "__main__":
    main()