from datetime import datetime
import os
from dataclasses import dataclass
from app.logger import log

from app.common import delete_file, download_file, load_yaml
from app.task import save_task_status


@dataclass
class TaskConfig:
    task_id: str
    themes: list
    software_list: list
    mode: str
    overwrite: bool = False
    sources: dict = None


def manage_tools(task_config: TaskConfig):
    start_time = datetime.now().isoformat()
    save_task_status(task_config.task_id, "running", start_time=start_time)
    yaml_data = load_yaml("repo/meta.yml")

    try:
        if not task_config.themes:
            task_config.themes = list(yaml_data.keys())

        for theme in task_config.themes:
            theme_data = yaml_data.get(theme)
            if not theme_data:
                continue

            for software in task_config.software_list:
                process_software(theme_data, software, task_config)

        end_time = datetime.now().isoformat()
        save_task_status(
            task_config.task_id, "completed", start_time=start_time, end_time=end_time
        )

    except Exception as e:
        end_time = datetime.now().isoformat()
        save_task_status(
            task_config.task_id,
            f"failed: {str(e)}",
            start_time=start_time,
            end_time=end_time,
        )
        log.error(f"Error: {e}")


def process_software(theme_data, software, task_config: TaskConfig):
    tool = software["name"]
    tool_data = theme_data.get(tool)
    if not tool_data:
        return

    archs = software.get("archs", [])
    versions = software.get("versions", [])

    for arch in archs:
        arch_data = tool_data.get(arch)
        if not arch_data:
            continue

        applicable_versions = versions if versions else list(arch_data.keys())
        for version in applicable_versions:
            version_data = arch_data.get(version)
            if not version_data:
                continue

            for file_info in version_data:
                process_file(tool, arch, version, file_info, task_config)


def process_file(tool, arch, version, file_info, task_config: TaskConfig):
    name = file_info["name"]
    url = file_info["source"]
    dest_dir = f"repo/{tool}/{arch}/{version}"
    os.makedirs(dest_dir, exist_ok=True)
    dest_path = os.path.join(dest_dir, name)

    overwrite_flag = task_config.overwrite

    if task_config.sources and isinstance(task_config.sources, dict):
        if (
            tool in task_config.sources
            and arch in task_config.sources[tool]
            and version in task_config.sources[tool][arch]
        ):
            url = task_config.sources[tool][arch][version]
            overwrite_flag = True

    if task_config.mode == "download":
        download_file(url, dest_path, overwrite=overwrite_flag)
    elif task_config.mode == "remove":
        delete_file(dest_path)
