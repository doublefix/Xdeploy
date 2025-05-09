use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::env;
use tokio::task;
use uuid::Uuid;

// export PYTHON_SYS_EXECUTABLE=$(pwd)/venv/bin/python
// export LD_LIBRARY_PATH=$(pwd)/venv/lib:$LD_LIBRARY_PATH
// export PRIVATE_DATA_DIR=/tmp/ansible_runner

#[derive(Debug, Clone)]
struct AnsibleRunParams {
    private_data_dir: String,
    playbook: String,
    cmd: Vec<String>,
    ident: String,
    verbosity: Option<i32>,
    quiet: Option<bool>,
}

impl AnsibleRunParams {
    fn new(
        private_data_dir: String,
        playbook: String,
        cmd: Vec<String>,
        verbosity: Option<i32>,
        quiet: Option<bool>,
    ) -> Self {
        Self {
            private_data_dir,
            playbook,
            cmd,
            ident: Uuid::new_v4().to_string(),
            verbosity,
            quiet,
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
        if let Some(verbosity) = self.verbosity {
            kwargs.set_item("verbosity", verbosity)?;
        }
        if let Some(quiet) = self.quiet {
            kwargs.set_item("quiet", quiet)?;
        }

        Ok(kwargs)
    }
}

async fn run_ansible(params: AnsibleRunParams) -> PyResult<()> {
    task::spawn_blocking(move || {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let ansible_runner = py.import("ansible_runner")?;

            println!("Running ansible with ident: {}", params.ident);

            let kwargs = params.to_py_dict(py)?;
            let result: Bound<'_, PyAny> =
                ansible_runner.call_method("run_async", (), Some(&kwargs))?;

            let thread: &Bound<'_, PyAny> = &result.get_item(0)?;
            thread.call_method0("join")?;

            Ok(())
        })
    })
    .await
    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task failed: {e}")))?
}

#[tokio::test]
async fn test_ansible_pyo3() -> PyResult<()> {
    let params = AnsibleRunParams::new(
        env::var("PRIVATE_DATA_DIR").expect("PRIVATE_DATA_DIR not set"),
        "playbooks/cmd.yml".to_string(),
        vec!["echo".to_string(), "Hello".to_string(), "World".to_string()],
        Some(1),
        None,
    );

    run_ansible(params).await
}
