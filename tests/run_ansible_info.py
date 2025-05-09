import ansible_runner

# 加载已有的 run 结果
r = ansible_runner.run(
    private_data_dir=".",
    playbook="playbooks/cmd.yml",
    inventory="inventory/hosts",
    ident="test-run"
)

# 打印状态和返回码
print(f"Status: {r.status}")
print(f"RC: {r.rc}")

# 输出事件日志（按步骤打印 stdout）
for ev in r.events:
    if "stdout" in ev:
        print(ev["stdout"])


# ansible-runner run . \
#   --inventory inventory/test \
#   --playbook playbooks/cmd.yml \
#   --ident test-run \
#   -v