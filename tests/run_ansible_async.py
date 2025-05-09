import ansible_runner
import time

# 启动异步 Ansible Runner
thread, r = ansible_runner.run_async(
    private_data_dir=".",
    playbook="playbooks/cmd.yml",
    extravars={"cmd": ["echo", "Hello", "World"]},
    verbosity=1,
)

print("Ansible playbook 正在后台运行...")

# 可以在这里做别的事情，例如轮询任务状态
while thread.is_alive():
    print("任务还在运行中...")
    time.sleep(1)

# 等待任务结束
thread.join()

# 打印状态和结果
print(f"\nStatus: {r.status}")
print(f"RC: {r.rc}")

print("\n--- STDOUT ---")
for ev in r.events:
    if "stdout" in ev:
        print(ev["stdout"])
