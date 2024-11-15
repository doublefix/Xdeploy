import os
import urllib
import yaml


def load_yaml(file_path):
    """Load a YAML file and return its content."""
    with open(file_path, "r") as file:
        return yaml.safe_load(file)


def download_file(url, dest_path, overwrite=False):
    """Download a file from the given URL to the specified destination path.

    Args:
        url (str): The URL of the file to download.
        dest_path (str): The destination path where the file should be saved.
        overwrite (bool): Whether to overwrite the file if it already exists. Default is False.
    """
    if os.path.exists(dest_path) and not overwrite:
        print(f"File {dest_path} already exists. Skipping download.")
        return

    print(f"Downloading from {url} to {dest_path}...")
    urllib.request.urlretrieve(url, dest_path)
    print("Download completed.")


def delete_file(dest_path):
    """Delete a file at the specified path if it exists.

    Args:
        dest_path (str): The path of the file to delete.
    """
    if os.path.exists(dest_path):
        os.remove(dest_path)
        print(f"Deleted file {dest_path}.")
    else:
        print(f"File {dest_path} does not exist. Skipping deletion.")
