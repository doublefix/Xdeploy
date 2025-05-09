use pyo3::prelude::*;
use pyo3::types::PyDict;

#[test]
fn test_ansible_pyo3() -> PyResult<()> {
    pyo3::prepare_freethreaded_python();
    // 初始化 Python 解释器
    Python::with_gil(|py| {
        // 导入需要的 Python 模块
        let uuid = py.import("uuid")?;
        let ansible_runner = py.import("ansible_runner")?;

        // 生成 UUID
        let run_uuid: String = uuid
            .call_method1("uuid4", ())?
            .call_method0("__str__")?
            .extract()?;
        println!("Generated UUID for this run: {run_uuid}",);

        // 准备参数
        let kwargs = PyDict::new(py);
        kwargs.set_item("private_data_dir", ".")?;
        kwargs.set_item("playbook", "../playbooks/cmd.yml")?;

        // 创建 extravars 字典
        let extravars = PyDict::new(py);
        let cmd = vec!["echo", "Hello", "World"];
        extravars.set_item("cmd", cmd)?;
        kwargs.set_item("extravars", extravars)?;

        kwargs.set_item("ident", run_uuid)?;
        kwargs.set_item("verbosity", 1)?;

        // 运行 Ansible Runner
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
