import os
import urllib.request
import yaml

TOOLS_TO_DOWNLOAD = ["kubelet"]  # 可设置为 ["kubectl"], ["kubeadm"], 或空列表 []
ARCHS_TO_DOWNLOAD = ["x86_64", "arrach64"]
MODE = "download"  # 可设置为 "download" 或 "remove" 

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

def delete_file(dest_path):
    if os.path.exists(dest_path):
        os.remove(dest_path)
        print(f"Deleted file {dest_path}")
    else:
        print(f"File {dest_path} does not exist. Skipping deletion.")

def main(overwrite=False):
    yaml_data = load_yaml("meta.yml")

    for tool_name, arch_data in yaml_data['kubernetes'].items():
        if tool_name not in TOOLS_TO_DOWNLOAD:
            continue

        for arch in ARCHS_TO_DOWNLOAD:
            if arch not in arch_data:
                print(f"Architecture {arch} not supported for tool {tool_name}. Skipping...")
                continue
            
            for version, files in arch_data[arch].items():
                for file_info in files:
                    name = file_info['name']
                    url = file_info['source']
                    
                    dest_dir = f"roles/{tool_name}/release/{arch}/{version}"
                    os.makedirs(dest_dir, exist_ok=True)
                    dest_path = os.path.join(dest_dir, name)

                    if MODE == "download":
                        download_file(url, dest_path, overwrite=overwrite)
                    elif MODE == "remove":
                        delete_file(dest_path)

if __name__ == "__main__":
    main(overwrite=False)
