import ansible_runner

project_dir = '/path'
inventory_path = 'inventory'

runner = ansible_runner.run(
    private_data_dir=project_dir,
    playbook='playbooks/nerdctl.yml',
    inventory=inventory_path
)

print("Status:", runner.status)
print("RC:", runner.rc)
print("Events:", runner.events)
print("Playbook stdout:", runner.stdout)