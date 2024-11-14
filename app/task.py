from datetime import datetime
import json
import os
import shutil

import ansible_runner


TASKS_DIR = "tasks"
MAX_TASKS = 3


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
