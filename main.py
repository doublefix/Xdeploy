from flask import Flask, request, jsonify
import ansible_runner
import os
import shutil
import tempfile
import threading

app = Flask(__name__)

task_status = {}

def run_playbook_task(task_id, playbook_path, inventory_path, extra_vars):
    try:
        tmp_dir = 'tmp_ansible_runner'
        os.makedirs(tmp_dir, exist_ok=True)

        runner = ansible_runner.run(
            private_data_dir=tmp_dir,
            playbook=playbook_path,
            inventory=inventory_path,
            extravars=extra_vars,
            roles_path=os.path.join(os.getcwd(), 'roles')
        )

        task_status[task_id] = 'success' if runner.rc == 0 else 'failure'

    except Exception as e:
        task_status[task_id] = f'failure: {str(e)}'
    finally:
        if os.path.exists(tmp_dir):
            shutil.rmtree(tmp_dir)
        if os.path.isfile(inventory_path):
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

    task_id = str(len(task_status) + 1)
    task_status[task_id] = 'running'

    thread = threading.Thread(target=run_playbook_task, args=(task_id, playbook_path, inventory_path, extra_vars))
    thread.start()

    return jsonify({'task_id': task_id, 'status': 'started'}), 202

@app.route('/task-status/<task_id>', methods=['GET'])
def get_task_status(task_id):
    if task_id in task_status:
        return jsonify({'task_id': task_id, 'status': task_status[task_id]}), 200
    else:
        return jsonify({'error': 'Task ID not found'}), 404

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
