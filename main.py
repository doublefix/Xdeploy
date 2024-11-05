import shutil
from flask import Flask, request, jsonify
import ansible_runner
import os
import tempfile
import threading
import uuid
import json
from datetime import datetime

app = Flask(__name__)

TASKS_DIR = 'tasks'
MAX_TASKS = 3

os.makedirs(TASKS_DIR, exist_ok=True)

def save_task_status(task_id, status, start_time=None, end_time=None):
    task_dirs = sorted(os.listdir(TASKS_DIR), key=lambda x: os.path.getctime(os.path.join(TASKS_DIR, x)))
    if len(task_dirs) > MAX_TASKS:
        oldest_task_dir = task_dirs[0]
        shutil.rmtree(os.path.join(TASKS_DIR, oldest_task_dir))

    task_dir = os.path.join(TASKS_DIR, task_id)
    os.makedirs(task_dir, exist_ok=True)
    
    task_info = {
        'status': status,
        'start_time': start_time,
        'end_time': end_time
    }
    with open(os.path.join(task_dir, 'status.json'), 'w') as f:
        json.dump(task_info, f)

def load_task_status(task_id):
    task_dir = os.path.join(TASKS_DIR, task_id)
    status_file = os.path.join(task_dir, 'status.json')
    if os.path.isfile(status_file):
        with open(status_file, 'r') as f:
            return json.load(f)
    return None

def run_playbook_task(task_id, playbook_path, inventory_path, extra_vars):
    start_time = datetime.now().isoformat()
    try:
        tmp_dir = os.path.join(TASKS_DIR, task_id, 'ansible_runner')
        os.makedirs(tmp_dir, exist_ok=True)

        runner = ansible_runner.run(
            private_data_dir=tmp_dir,
            playbook=playbook_path,
            inventory=inventory_path,
            extravars=extra_vars,
            roles_path=os.path.join(os.getcwd(), 'roles')
        )

        status = 'success' if runner.rc == 0 else 'failure'
        end_time = datetime.now().isoformat()
        save_task_status(task_id, status, start_time, end_time)

    except Exception as e:
        end_time = datetime.now().isoformat()
        save_task_status(task_id, f'failure: {str(e)}', start_time, end_time)
    finally:
        if os.path.exists(inventory_path):
            os.remove(inventory_path)

@app.route('/run-playbook', methods=['POST'])
def run_playbook():
    data = request.json
    playbook_path = data.get('playbook_path', 'playbooks/playbook.yml')
    inventory_data = data.get('inventory', {})
    extra_vars = data.get('extra_vars', {})

    current_dir = os.getcwd()
    playbook_path = os.path.join(current_dir, playbook_path)

    if not os.path.isfile(playbook_path):
        return jsonify({'error': f'Playbook not found: {playbook_path}'}), 400

    with tempfile.NamedTemporaryFile(delete=False, mode='w', suffix='.ini') as inventory_file:
        inventory_file.write("[servers]\n")
        for host in inventory_data.get('servers', {}).get('hosts', []):
            inventory_file.write(f"{host}\n")
        inventory_path = inventory_file.name

    task_id = str(uuid.uuid4())
    save_task_status(task_id, 'running')

    thread = threading.Thread(target=run_playbook_task, args=(task_id, playbook_path, inventory_path, extra_vars))
    thread.start()

    return jsonify({'task_id': task_id, 'status': 'started'}), 202

@app.route('/task-status/<task_id>', methods=['GET'])
def get_task_status(task_id):
    task_info = load_task_status(task_id)
    if task_info:
        return jsonify({'task_id': task_id, **task_info}), 200
    else:
        return jsonify({'error': 'Task ID not found'}), 404

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
