use std::env;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use uuid::Uuid;

// export PYTHON_SYS_EXECUTABLE=$(pwd)/venv/bin/python
// export LD_LIBRARY_PATH=$(pwd)/venv/lib:$LD_LIBRARY_PATH
// export PRIVATE_DATA_DIR=/tmp/ansible_runner

#[derive(Debug)]
struct AnsibleRunParams {
    private_data_dir: String,
    playbook: String,
    cmd: Vec<String>,
    ident: String,
    verbosity: i32,
}

impl AnsibleRunParams {
    fn new(private_data_dir: String, playbook: String, cmd: Vec<String>) -> Self {
        Self {
            private_data_dir,
            playbook,
            cmd,
            ident: Uuid::new_v4().to_string(),
            verbosity: 1,
        }
    }

    fn to_py_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("private_data_dir", &self.private_data_dir)?;
        kwargs.set_item("playbook", &self.playbook)?;

        let extravars = PyDict::new(py);
        extravars.set_item("cmd", &self.cmd)?;
        kwargs.set_item("extravars", extravars)?;

        kwargs.set_item("ident", &self.ident)?;
        kwargs.set_item("verbosity", self.verbosity)?;

        Ok(kwargs)
    }
}

#[test]
fn test_ansible_pyo3() -> PyResult<()> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let ansible_runner = py.import("ansible_runner")?;

        // 使用结构体封装参数
        let params = AnsibleRunParams::new(
            env::var("PRIVATE_DATA_DIR").expect("Environment variable 'private_data_dir' not set"),
            "playbooks/cmd.yml".to_string(),
            vec!["echo".to_string(), "Hello".to_string(), "World".to_string()],
        );

        println!("Ident: {}", params.ident);

        // 运行 Ansible Runner
        let kwargs = params.to_py_dict(py)?;
        let r = ansible_runner.call_method("run", (), Some(&kwargs))?;

        // 获取并打印结果
        let status: String = r.getattr("status")?.extract()?;
        let rc: i32 = r.getattr("rc")?.extract()?;
        println!("Status: {status}");
        println!("RC: {rc}");

        // 打印 stdout 事件
        println!("\n--- STDOUT ---");
        let events = r.getattr("events")?;
        for event in events.try_iter()? {
            let event = event?;
            if let Ok(stdout) = event.get_item("stdout") {
                if !stdout.is_none() {
                    println!("{stdout}");
                }
            }
        }

        Ok(())
    })
}
