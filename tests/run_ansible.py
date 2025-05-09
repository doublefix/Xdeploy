# https://ansible.readthedocs.io/projects/runner/en/latest/standalone/
import uuid
import ansible_runner

run_uuid = str(uuid.uuid4())
print(f"Generated UUID for this run: {run_uuid}")

r = ansible_runner.run(
    private_data_dir=".",
    playbook="playbooks/cmd.yml",
    extravars={"cmd": ["echo", "Hello", "World"]},
    ident=run_uuid,
    quiet=True,
    # verbosity=1,
)

# 打印状态和事件输出
# print(f"Status: {r.status}")
# print(f"RC: {r.rc}")

# print("\n--- STDOUT ---")
# for ev in r.events:
#     if "stdout" in ev:
#         print(ev["stdout"])
