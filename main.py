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


def manage_tools(task_id, tools, archs, versions, mode, overwrite=False):
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    yaml_data = load_yaml("meta.yml")

    try:
        for tool_name, arch_data in yaml_data["kubernetes"].items():
            if tool_name not in tools:
                continue

            for arch in archs:
                if arch not in arch_data:
                    print(f"架构 {arch} 不支持工具 {tool_name}，跳过...")
                    continue

                # 根据传入的版本进行筛选，如果 versions 为空则使用所有版本
                for version, files in arch_data[arch].items():
                    if versions and version not in versions:
                        continue

                    for file_info in files:
                        name = file_info["name"]
                        url = file_info["source"]

                        dest_dir = f"roles/{tool_name}/release/{arch}/{version}"
                        os.makedirs(dest_dir, exist_ok=True)
                        dest_path = os.path.join(dest_dir, name)

                        if mode == "download":
                            download_file(url, dest_path, overwrite=overwrite)
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
    theme = data.get("theme")
    tools = data.get("tools")
    archs = data.get("archs")
    versions = data.get("versions")
    mode = data.get("mode")
    overwrite = data.get("overwrite")

    if (
        not theme
        or not tools
        or not archs
        or not versions
        or mode is None
        or overwrite is None
    ):
        return make_response({"error": "Missing required fields."}, 400)

    yaml_data = load_yaml("meta.yml")

    if theme not in yaml_data:
        return make_response({"error": f"'{theme}' configuration not found"}, 400)

    # 根据 theme 获取对应的工具配置
    theme_data = yaml_data[theme]
    unsupported = []

    for tool in tools:
        if tool not in theme_data:
            unsupported.append(f"Tool '{tool}' is not supported.")
            continue

        for arch in archs:
            if arch not in theme_data[tool]:
                unsupported.append(
                    f"Tool '{tool}' does not support architecture '{arch}'."
                )
                continue

            applicable_versions = (
                versions if versions else list(theme_data[tool][arch].keys())
            )

            for version in applicable_versions:
                if version not in theme_data[tool][arch]:
                    unsupported.append(
                        f"Tool '{tool}' with architecture '{arch}' does not support version '{version}'."
                    )

    if unsupported:
        return make_response(
            {
                "error": "Task creation failed due to unsupported configuration",
                "details": unsupported,
            },
            400,
        )

    task_id = str(uuid.uuid4())
    start_time = datetime.now().isoformat()
    save_task_status(task_id, "running", start_time=start_time)

    thread = threading.Thread(
        target=manage_tools, args=(task_id, tools, archs, versions, mode, overwrite)
    )
    thread.start()

    return jsonify({"task_id": task_id, "status": "started"}), 202


@app.route("/run-playbook", methods=["POST"])
def run_playbook():
    data = request.json
    playbook_path = data.get("playbook_path", "playbooks/playbook.yml")
    inventory_data = data.get("inventory", {})
    extra_vars = data.get("extra_vars", {})

    current_dir = os.getcwd()
    playbook_path = os.path.join(current_dir, playbook_path)

    if not os.path.isfile(playbook_path):
        return jsonify({"error": f"Playbook not found: {playbook_path}"}), 400

    with tempfile.NamedTemporaryFile(
        delete=False, mode="w", suffix=".ini"
    ) as inventory_file:
        inventory_file.write("[servers]\n")
        for host in inventory_data.get("servers", {}).get("hosts", []):
            inventory_file.write(f"{host}\n")
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
