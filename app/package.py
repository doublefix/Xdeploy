from datetime import datetime
import os

from app.common import delete_file, download_file, load_yaml
from app.task import save_task_status


def manage_tools(task_id, themes, software_list, mode, overwrite=False, sources={}):
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)
    yaml_data = load_yaml("meta.yml")

    try:
        for theme in themes:
            theme_data = yaml_data.get(theme)
            if not theme_data:
                continue

            for software in software_list:
                tool = software["name"]
                archs = software.get("archs", [])
                versions = software.get("versions", [])
                tool_data = theme_data.get(tool)
                if not tool_data:
                    continue

                for arch in archs:
                    arch_data = tool_data.get(arch)
                    if not arch_data:
                        continue

                    applicable_versions = (
                        versions if versions else list(arch_data.keys())
                    )

                    for version in applicable_versions:
                        version_data = arch_data.get(version)
                        if not version_data:
                            continue

                        for file_info in version_data:
                            name = file_info["name"]
                            url = file_info["source"]

                            if (
                                tool in sources
                                and arch in sources[tool]
                                and version in sources[tool][arch]
                            ):
                                url = sources[tool][arch][version]
                                overwrite_flag = True
                            else:
                                overwrite_flag = overwrite

                            dest_dir = f"roles/{tool}/release/{arch}/{version}"
                            os.makedirs(dest_dir, exist_ok=True)
                            dest_path = os.path.join(dest_dir, name)

                            if mode == "download":
                                download_file(url, dest_path, overwrite=overwrite_flag)
                            elif mode == "remove":
                                delete_file(dest_path)

        end_time = datetime.now().isoformat()
        save_task_status(task_id, "completed", start_time=start_time, end_time=end_time)

    except Exception as e:
        end_time = datetime.now().isoformat()
        save_task_status(
            task_id, f"failed: {str(e)}", start_time=start_time, end_time=end_time
        )
        print(f"Error: {e}")
