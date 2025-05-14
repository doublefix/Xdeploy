use pyo3::prelude::*;
use pyo3::types::PyDict;
use uuid::Uuid;
pub mod ansible {
    tonic::include_proto!("ansible");
}

pub use ansible::{
    AnsibleGroup, AnsibleInventory, AnsibleTaskStatusRequest, AnsibleTaskStatusResponse,
    DeployRequest, DeployResponse,
};

// export PYTHON_SYS_EXECUTABLE=$(pwd)/venv/bin/python
// export LD_LIBRARY_PATH=$(pwd)/venv/lib:$LD_LIBRARY_PATH
// export PRIVATE_DATA_DIR=$(pwd)
// #[derive(Debug, Clone)]
pub struct AnsibleRunParams {
    private_data_dir: String,
    playbook: String,
    inventory: AnsibleInventory,
    cmd: Vec<String>,
    ident: String,
    verbosity: Option<i32>,
    quiet: Option<bool>,
}

// 第一阶段：必须设置的参数
pub struct RequiredParams {
    private_data_dir: String,
    playbook: String,
    inventory: AnsibleInventory,
}

impl RequiredParams {
    pub fn new(
        private_data_dir: impl Into<String>,
        playbook: impl Into<String>,
        inventory: AnsibleInventory,
    ) -> Self {
        Self {
            private_data_dir: private_data_dir.into(),
            playbook: playbook.into(),
            inventory,
        }
    }

    pub fn with_cmd<S: AsRef<str>>(self, cmd: impl Into<Vec<S>>) -> ParamsWithCmd {
        ParamsWithCmd {
            private_data_dir: self.private_data_dir,
            playbook: self.playbook,
            inventory: self.inventory,
            cmd: cmd
                .into()
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        }
    }
}

// 第二阶段：设置cmd参数
pub struct ParamsWithCmd {
    private_data_dir: String,
    playbook: String,
    cmd: Vec<String>,
    inventory: AnsibleInventory,
}

impl ParamsWithCmd {
    pub fn build(self) -> AnsibleRunParams {
        AnsibleRunParams {
            private_data_dir: self.private_data_dir,
            playbook: self.playbook,
            cmd: self.cmd,
            ident: Uuid::new_v4().to_string(),
            verbosity: None,
            quiet: None,
            inventory: self.inventory,
        }
    }

    pub fn with_optional(self) -> OptionalParams {
        OptionalParams {
            private_data_dir: self.private_data_dir,
            playbook: self.playbook,
            inventory: self.inventory,
            cmd: self.cmd,
            ident: None,
            verbosity: None,
            quiet: None,
        }
    }
}

// 第三阶段：可选参数
pub struct OptionalParams {
    private_data_dir: String,
    playbook: String,
    inventory: AnsibleInventory,
    cmd: Vec<String>,
    ident: Option<String>,
    verbosity: Option<i32>,
    quiet: Option<bool>,
}

impl OptionalParams {
    pub fn ident(mut self, ident: impl Into<String>) -> Self {
        self.ident = Some(ident.into());
        self
    }

    pub fn verbosity(mut self, verbosity: i32) -> Self {
        self.verbosity = Some(verbosity);
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = Some(quiet);
        self
    }

    pub fn build(self) -> AnsibleRunParams {
        AnsibleRunParams {
            private_data_dir: self.private_data_dir,
            playbook: self.playbook,
            cmd: self.cmd,
            ident: self.ident.unwrap_or_else(|| Uuid::new_v4().to_string()),
            verbosity: self.verbosity,
            quiet: self.quiet,
            inventory: self.inventory,
        }
    }
}

impl AnsibleRunParams {
    pub fn builder(
        private_data_dir: impl Into<String>,
        playbook: impl Into<String>,
        inventory: AnsibleInventory,
    ) -> RequiredParams {
        RequiredParams::new(private_data_dir, playbook, inventory)
    }

    fn to_py_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("private_data_dir", &self.private_data_dir)?;
        kwargs.set_item("playbook", &self.playbook)?;
        let inventory_dict = convert_inventory(py, &self.inventory)?;
        kwargs.set_item("inventory", inventory_dict)?;

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

fn convert_inventory<'py>(py: Python<'py>, inv: &AnsibleInventory) -> PyResult<Bound<'py, PyDict>> {
    let inventory_dict = PyDict::new(py);
    let all = PyDict::new(py);
    let children = PyDict::new(py);

    for (group_name, group) in &inv.children {
        let group_dict = convert_group(py, group)?;
        children.set_item(group_name, group_dict)?;
    }

    all.set_item("children", children)?;
    inventory_dict.set_item("all", all)?;
    Ok(inventory_dict)
}

fn convert_group<'py>(py: Python<'py>, group: &AnsibleGroup) -> PyResult<Bound<'py, PyDict>> {
    let group_dict = PyDict::new(py);

    // hosts
    let hosts_dict = PyDict::new(py);
    for (host_name, host) in &group.hosts {
        let host_dict = PyDict::new(py);
        let vars_dict = PyDict::new(py);
        for (k, v) in &host.vars {
            vars_dict.set_item(k, v)?;
        }
        host_dict.set_item("vars", vars_dict)?;
        hosts_dict.set_item(host_name, host_dict)?;
    }
    group_dict.set_item("hosts", hosts_dict)?;

    // vars
    let vars_dict = PyDict::new(py);
    for (k, v) in &group.vars {
        vars_dict.set_item(k, v)?;
    }
    group_dict.set_item("vars", vars_dict)?;

    // children
    let children_dict = PyDict::new(py);
    for (child_name, child_group) in &group.children {
        let child_group_dict = convert_group(py, child_group)?;
        children_dict.set_item(child_name, child_group_dict)?;
    }
    group_dict.set_item("children", children_dict)?;

    Ok(group_dict)
}

pub async fn run_ansible(params: AnsibleRunParams) -> PyResult<()> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let ansible_runner = py.import("ansible_runner")?;
        let kwargs = params.to_py_dict(py)?;

        ansible_runner.call_method("run_async", (), Some(&kwargs))?;
        Ok(())
    })
}
