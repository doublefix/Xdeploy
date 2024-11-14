from datetime import datetime
import os
import tempfile
import threading
import uuid
from flask import Blueprint, app, jsonify, make_response, request

from app.common import load_yaml
from app.package import manage_tools
from app.task import load_task_status, run_playbook_task, save_task_status

routes = Blueprint("routes", __name__)


@routes.route("/manage-tools", methods=["POST"])
def manage_tools_endpoint():
    data = request.json
    themes = data.get("themes", [])
    software_list = data.get("software", [])
    mode = data.get("mode")
    overwrite = data.get("overwrite", False)
    sources = data.get("sources", {})

    if not software_list or not mode:
        return make_response({"error": "缺少必要的字段"}, 400)

    yaml_data = load_yaml("meta.yml")
    if not themes:
        themes = list(yaml_data.keys())

    unsupported = []
    for software in software_list:
        tool = software["name"]
        archs = software.get("archs", [])
        versions = software.get("versions", [])
        tool_supported = False

        for theme in themes:
            theme_data = yaml_data.get(theme)
            if not theme_data:
                continue

            tool_data = theme_data.get(tool)
            if not tool_data:
                continue

            for arch in archs:
                arch_data = tool_data.get(arch)
                if not arch_data:
                    continue

                applicable_versions = versions if versions else list(arch_data.keys())

                for version in applicable_versions:
                    if version in arch_data:
                        tool_supported = True
                        break
                if tool_supported:
                    break
            if tool_supported:
                break

        if not tool_supported:
            unsupported.append(
                f"未找到支持的主题配置：工具 '{tool}', 架构 '{archs}', 版本 '{versions}'"
            )

    if unsupported:
        return make_response(
            {
                "error": "任务创建失败，配置不受支持",
                "details": unsupported,
            },
            400,
        )

    task_id = str(uuid.uuid4())
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    thread = threading.Thread(
        target=manage_tools,
        args=(task_id, themes, software_list, mode, overwrite, sources),
    )
    thread.start()

    return jsonify({"task_id": task_id, "status": "started"}), 202


@routes.route("/manage-all-themes", methods=["POST"])
def manage_all_themes():
    data = request.json
    overwrite = data.get("overwrite", False)
    mode = data.get("mode", "download")

    yaml_data = load_yaml("meta.yml")
    themes = list(yaml_data.keys())
    software_list = []

    for theme in themes:
        theme_data = yaml_data.get(theme)
        if theme_data:
            for tool_name, tool_data in theme_data.items():
                software = {
                    "name": tool_name,
                    "archs": list(tool_data.keys()),
                    "versions": [],
                }
                for arch, arch_data in tool_data.items():
                    software["versions"].extend(list(arch_data.keys()))
                software_list.append(software)

    task_id = str(uuid.uuid4())
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    thread = threading.Thread(
        target=manage_tools,
        args=(task_id, themes, software_list, mode, overwrite),
    )
    thread.start()

    return jsonify({"task_id": task_id, "status": "started"}), 202


@routes.route("/run-playbook", methods=["POST"])
def run_playbook():
    data = request.json
    playbook_path = data.get("playbook", "playbooks/playbook.yml")
    inventory_data = data.get("inventory", {})
    extra_vars = data.get("extra_vars", {})

    current_dir = os.getcwd()
    playbook_path = os.path.join(current_dir, playbook_path)

    if not os.path.isfile(playbook_path):
        return jsonify({"error": f"Playbook not found: {playbook_path}"}), 400

    servers = inventory_data.get("servers", [])
    if not isinstance(servers, list):
        return (
            jsonify({"error": "Invalid inventory format, 'servers' should be a list"}),
            400,
        )

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
                return (
                    jsonify(
                        {"error": "Each server must have 'host' and 'user' fields"}
                    ),
                    400,
                )
            host = server.get("host")
            user = server.get("user")
            inventory_file.write(f"{host} ansible_user={user}\n")
        inventory_path = inventory_file.name

    task_id = str(uuid.uuid4())
    save_task_status(task_id, "running")

    thread = threading.Thread(
        target=run_playbook_task,
        args=(task_id, playbook_path, inventory_path, extra_vars),
    )
    thread.start()

    return jsonify({"task_id": task_id, "status": "started"}), 202


@routes.route("/task-status/<task_id>", methods=["GET"])
def get_task_status(task_id):
    task_info = load_task_status(task_id)
    if task_info:
        return jsonify({"task_id": task_id, **task_info}), 200
    else:
        return jsonify({"error": "Task ID not found"}), 404


@routes.route("/health-check", methods=["GET"])
def health_check():
    return jsonify({"status": "ok", "message": "Service is up and running"}), 200
