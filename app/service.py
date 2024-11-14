import os
import tempfile
import threading
from datetime import datetime
import uuid
from app.package import TaskConfig, manage_tools
from app.task import save_task_status


def extract_request_data(data):
    """提取并处理请求数据的默认值"""
    themes = data.get("themes", [])
    software_list = data.get("software", [])
    mode = data.get("mode")
    overwrite = data.get("overwrite", False)
    sources = data.get("sources", {})
    return themes, software_list, mode, overwrite, sources


def check_supported_tools(themes, software_list, yaml_data):
    """检查请求的工具配置是否受支持"""
    unsupported = []

    for software in software_list:
        tool, archs, versions = (
            software["name"],
            software.get("archs", []),
            software.get("versions", []),
        )
        if not is_tool_supported(tool, archs, versions, themes, yaml_data):
            unsupported.append(
                f"未找到支持的主题配置：工具 '{tool}', 架构 '{archs}', 版本 '{versions}'"
            )

    return unsupported


def is_tool_supported(tool, archs, versions, themes, yaml_data):
    """判断特定工具是否支持特定架构和版本"""
    for theme in themes:
        theme_data = yaml_data.get(theme)
        tool_data = theme_data.get(tool) if theme_data else None
        if not tool_data:
            continue

        for arch in archs:
            arch_data = tool_data.get(arch)
            if not arch_data:
                continue

            applicable_versions = versions if versions else list(arch_data.keys())
            if any(version in arch_data for version in applicable_versions):
                return True
    return False


def start_task_job(task_id, themes, software_list, mode, overwrite, sources):
    """初始化并启动任务线程"""
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    task_config = TaskConfig(
        task_id=task_id,
        themes=themes,
        software_list=software_list,
        mode=mode,
        overwrite=overwrite,
        sources=sources,
    )
    thread = threading.Thread(target=manage_tools, args=(task_config,))
    thread.start()


def generate_software_list(yaml_data, themes):
    """根据 YAML 数据生成包含所有主题的软件列表"""
    software_list = []
    for theme in themes:
        theme_data = yaml_data.get(theme)
        if not theme_data:
            continue

        for tool_name, tool_data in theme_data.items():
            software = {
                "name": tool_name,
                "archs": list(tool_data.keys()),
                "versions": [],
            }
            for arch, arch_data in tool_data.items():
                software["versions"].extend(list(arch_data.keys()))
            software_list.append(software)

    return software_list


def start_task(themes, software_list, mode, overwrite):
    """初始化任务配置并启动新线程"""
    task_id = str(uuid.uuid4())
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    task_config = TaskConfig(
        task_id=task_id,
        themes=themes,
        software_list=software_list,
        mode=mode,
        overwrite=overwrite,
    )
    thread = threading.Thread(target=manage_tools, args=(task_config,))
    thread.start()

    return task_id


def extract_playbook_data(data):
    """Extract playbook related data from the request"""
    playbook_path = data.get("playbook", "playbooks/playbook.yml")
    inventory_data = data.get("inventory", {})
    extra_vars = data.get("extra_vars", {})
    return playbook_path, inventory_data, extra_vars


def validate_playbook_path(playbook_path):
    """Validate and return the full playbook path"""
    current_dir = os.getcwd()
    full_playbook_path = os.path.join(current_dir, playbook_path)
    if not os.path.isfile(full_playbook_path):
        raise FileNotFoundError(f"Playbook not found: {full_playbook_path}")
    return full_playbook_path


def generate_inventory_file(inventory_data):
    """Generate a temporary inventory file"""
    servers = inventory_data.get("servers", [])
    if not isinstance(servers, list):
        return None

    try:
        with tempfile.NamedTemporaryFile(
            delete=False, mode="w", suffix=".ini"
        ) as inventory_file:
            inventory_file.write("[servers]\n")
            for server in servers:
                if (
                    not isinstance(server, dict)
                    or "host" not in server
                    or "user" not in server
                ):
                    return None
                host = server["host"]
                user = server["user"]
                inventory_file.write(f"{host} ansible_user={user}\n")
            return inventory_file.name
    except Exception as e:
        return None
