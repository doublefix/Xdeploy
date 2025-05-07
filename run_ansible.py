import ansible_runner

r = ansible_runner.run(
    private_data_dir='.',                    # 当前目录（需要包含 inventory 文件或显式传）
    playbook='playbooks/cmd.yml',            # Playbook 路径
    extravars={"cmd": ["echo", "Hello", "World"]},
    verbosity=1
)

# 打印状态和事件输出
print(f"Status: {r.status}")
print(f"RC: {r.rc}")

print("\n--- STDOUT ---")
for ev in r.events:
    if 'stdout' in ev:
        print(ev['stdout'])