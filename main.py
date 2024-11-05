from flask import Flask, request, jsonify
import ansible_runner
import os
import shutil
import tempfile

app = Flask(__name__)

@app.route('/run-playbook', methods=['POST'])
def run_playbook():
    data = request.json
    playbook_path = data.get('playbook_path', 'playbooks/playbook.yml')
    inventory_data = data.get('inventory', {})  # 从请求中获取 inventory
    extra_vars = data.get('extra_vars', {})

    current_dir = os.getcwd()
    playbook_path = os.path.join(current_dir, playbook_path)

    # 调试输出
    print(f"Playbook path: {playbook_path}")

    if not os.path.isfile(playbook_path):
        return jsonify({'error': f'Playbook not found: {playbook_path}'}), 400

    # 创建临时 inventory 文件
    with tempfile.NamedTemporaryFile(delete=False, mode='w', suffix='.ini') as inventory_file:
        inventory_file.write("[servers]\n")  # 写入组头
        for host in inventory_data.get('servers', {}).get('hosts', []):
            inventory_file.write(f"{host}\n")  # 写入主机名
        inventory_path = inventory_file.name

    # 创建运行的临时目录
    tmp_dir = 'tmp_ansible_runner'
    os.makedirs(tmp_dir, exist_ok=True)

    try:
        # 运行 Ansible playbook
        runner = ansible_runner.run(
            private_data_dir=tmp_dir,
            playbook=playbook_path,
            inventory=inventory_path,
            extravars=extra_vars,
            roles_path=os.path.join(current_dir, 'roles')  # 指定 roles 路径
        )

        # 仅返回成功或失败的状态
        if runner.rc == 0:
            return jsonify({'status': 'success'}), 200
        else:
            return jsonify({'status': 'failure'}), 500

    except Exception as e:
        print(f"Exception occurred: {str(e)}")
        return jsonify({'status': 'failure', 'error': str(e)}), 500
    finally:
        # 清理临时目录和 inventory 文件
        if os.path.exists(tmp_dir):
            shutil.rmtree(tmp_dir)
        if os.path.isfile(inventory_path):
            os.remove(inventory_path)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)