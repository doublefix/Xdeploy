from datetime import datetime
import os
import tempfile
import threading
import uuid
from flask import Blueprint, app, jsonify, make_response, request

from app.common import load_yaml
from app.package import TaskConfig, manage_tools
from app.service import (
    check_supported_tools,
    extract_playbook_data,
    extract_request_data,
    generate_inventory_file,
    generate_software_list,
    start_task,
    start_task_job,
    validate_playbook_path,
)
from app.task import load_task_status, run_playbook_task, save_task_status

routes = Blueprint("routes", __name__)


@routes.route("/manage-tools", methods=["POST"])
def manage_tools_endpoint():
    data = request.json
    themes, software_list, mode, overwrite, sources = extract_request_data(data)

    if not software_list or not mode:
        return make_response({"error": "缺少必要的字段"}, 400)
    yaml_data = load_yaml("meta.yml")
    themes = themes or list(yaml_data.keys())

    unsupported = check_supported_tools(themes, software_list, yaml_data)
    if unsupported:
        return make_response(
            {
                "error": "任务创建失败，配置不受支持",
                "details": unsupported,
            },
            400,
        )
    task_id = str(uuid.uuid4())
    start_task_job(task_id, themes, software_list, mode, overwrite, sources)

    return jsonify({"task_id": task_id, "status": "started"}), 202


@routes.route("/manage-all-themes", methods=["POST"])
def manage_all_themes():
    data = request.json
    overwrite = data.get("overwrite", False)
    mode = data.get("mode", "download")

    yaml_data = load_yaml("meta.yml")
    themes = list(yaml_data.keys())
    software_list = generate_software_list(yaml_data, themes)

    task_id = start_task(themes, software_list, mode, overwrite)
    return jsonify({"task_id": task_id, "status": "started"}), 202


@routes.route("/run-playbook", methods=["POST"])
def run_playbook():
    data = request.json
    playbook_path, inventory_data, extra_vars = extract_playbook_data(data)

    playbook_path = validate_playbook_path(playbook_path)
    inventory_path = generate_inventory_file(inventory_data)

    if not inventory_path:
        return jsonify({"error": "Invalid inventory format"}), 400

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
