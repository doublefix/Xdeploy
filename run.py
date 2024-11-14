import shutil
from flask import Flask, make_response, request, jsonify
import ansible_runner
import os
import tempfile
import threading
import uuid
import json
from datetime import datetime
import urllib.request
import yaml

app = Flask(__name__)

TASKS_DIR = "tasks"
MAX_TASKS = 3

os.makedirs(TASKS_DIR, exist_ok=True)


def save_task_status(task_id, status, start_time=None, end_time=None):
    task_dirs = sorted(
        os.listdir(TASKS_DIR),
        key=lambda x: os.path.getctime(os.path.join(TASKS_DIR, x)),
    )
    if len(task_dirs) > MAX_TASKS:
        oldest_task_dir = task_dirs[0]
        shutil.rmtree(os.path.join(TASKS_DIR, oldest_task_dir))

    task_dir = os.path.join(TASKS_DIR, task_id)
    os.makedirs(task_dir, exist_ok=True)

    task_info = {"status": status, "start_time": start_time, "end_time": end_time}
    with open(os.path.join(task_dir, "status.json"), "w") as f:
        json.dump(task_info, f)


def load_task_status(task_id):
    task_dir = os.path.join(TASKS_DIR, task_id)
    status_file = os.path.join(task_dir, "status.json")
    if os.path.isfile(status_file):
        with open(status_file, "r") as f:
            return json.load(f)
    return None


def run_playbook_task(task_id, playbook_path, inventory_path, extra_vars):
    start_time = datetime.now().isoformat()
    try:
        tmp_dir = os.path.join(TASKS_DIR, task_id, "ansible_runner")
        os.makedirs(tmp_dir, exist_ok=True)

        runner = ansible_runner.run(
            private_data_dir=tmp_dir,
            playbook=playbook_path,
            inventory=inventory_path,
            extravars=extra_vars,
            roles_path=os.path.join(os.getcwd(), "roles"),
        )

        status = "success" if runner.rc == 0 else "failure"
        end_time = datetime.now().isoformat()
        save_task_status(task_id, status, start_time, end_time)

    except Exception as e:
        end_time = datetime.now().isoformat()
        save_task_status(task_id, f"failure: {str(e)}", start_time, end_time)
    finally:
        if os.path.exists(inventory_path):
            os.remove(inventory_path)


def load_yaml(file_path):
    with open(file_path, "r") as file:
        return yaml.safe_load(file)


def download_file(url, dest_path, overwrite=False):
    if os.path.exists(dest_path) and not overwrite:
        print(f"文件 {dest_path} 已存在，跳过下载。")
        return

    print(f"从 {url} 下载到 {dest_path}...")
    urllib.request.urlretrieve(url, dest_path)
    print("下载完成。")


def delete_file(dest_path):
    if os.path.exists(dest_path):
        os.remove(dest_path)
        print(f"已删除文件 {dest_path}")
    else:
        print(f"文件 {dest_path} 不存在，跳过删除。")


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


@app.route("/manage-tools", methods=["POST"])
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


@app.route("/manage-all-themes", methods=["POST"])
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


@app.route("/run-playbook", methods=["POST"])
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


@app.route("/task-status/<task_id>", methods=["GET"])
def get_task_status(task_id):
    task_info = load_task_status(task_id)
    if task_info:
        return jsonify({"task_id": task_id, **task_info}), 200
    else:
        return jsonify({"error": "Task ID not found"}), 404


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
