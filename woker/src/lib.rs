pub mod deploy;
pub use deploy::AnsibleRunParams;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod agent {
    tonic::include_proto!("api"); // proto package 名为 agent
}

pub mod ansible {
    tonic::include_proto!("ansible");
}

// 错误类型
type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

use ansible::{AnsibleTaskStatusRequest, AnsibleTaskStatusResponse, DeployRequest, DeployResponse};

mod types {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct HelloInput {
        pub name: String,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct HelloOutput {
        pub greeting: String,
        pub original: HelloInput,
    }
}
// 函数处理器模块
pub mod function_handlers {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::types::{HelloInput, HelloOutput};

    // 定义函数处理器特征
    pub trait FunctionHandler<I, O>: Send + Sync
    where
        I: for<'de> Deserialize<'de>,
        O: Serialize,
    {
        fn handle(&self, input: I) -> Result<O>;
    }

    // 函数注册表类型
    type HandlerMap =
        HashMap<String, Box<dyn FunctionHandler<serde_json::Value, serde_json::Value>>>;

    // 全局函数注册表
    pub static FUNCTION_REGISTRY: Lazy<HandlerMap> = Lazy::new(|| {
        let mut map: HandlerMap = HashMap::new();
        map.insert(
            "Hello".to_string(),
            Box::new(JsonFunctionWrapper::new(hello_handler)),
        );
        map.insert(
            "Deploy".to_string(),
            Box::new(JsonFunctionWrapper::new(deploy_handler)),
        );
        map.insert(
            "DeployStatus".to_string(),
            Box::new(JsonFunctionWrapper::new(deploy_status_handler)),
        );
        map
    });

    // 具体函数实现
    pub fn hello_handler(input: HelloInput) -> Result<HelloOutput> {
        Ok(HelloOutput {
            greeting: format!("Hello, {}", input.name),
            original: input,
        })
    }

    pub fn deploy_handler(input: DeployRequest) -> Result<DeployResponse> {
        println!("Deploying with request_id: {input:#?}");

        // 获取配置路径
        let private_data_dir = std::env::var("PRIVATE_DATA_DIR")
            .expect("PRIVATE_DATA_DIR environment variable not set");
        let task_ident = uuid::Uuid::new_v4().to_string();

        // 构建 Ansible 参数（保持你原有 builder 调用）
        if let Some(params) = input.params {
            let params_full = AnsibleRunParams::builder(private_data_dir, params.playbook)
                .with_cmd(params.cmd)
                .with_optional()
                .ident(task_ident.clone())
                .verbosity(1)
                // .quiet(true)
                .build();

            tokio::spawn(async move {
                if let Err(e) = deploy::run_ansible(params_full).await {
                    eprintln!("Ansible task failed: {e}");
                }
            });
        } else {
            println!("No deploy params provided");
        }

        let start_time = 21412;

        // 启动 ansible 异步任务

        Ok(DeployResponse {
            task_ident,
            start_time,
            initial_status: "scheduled".to_string(),
        })
    }

    pub fn deploy_status_handler(
        input: AnsibleTaskStatusRequest,
    ) -> Result<AnsibleTaskStatusResponse> {
        // 获取环境变量
        let private_data_dir = std::env::var("PRIVATE_DATA_DIR")
            .map_err(|e| anyhow::anyhow!("PRIVATE_DATA_DIR not set: {}", e))?;

        // let private_data_dir = "/code/Xdeploy/artifacts";
        let private_data_dir = std::path::PathBuf::from(private_data_dir).join("artifacts");

        // 构建并验证任务目录
        let task_dir = PathBuf::from(&private_data_dir).join(&input.ident);
        if !task_dir.exists() {
            return not_found_response(
                &input.ident,
                &format!("Task directory at {}", task_dir.display()),
            );
        }

        // 读取rc文件
        let rc = read_rc_file(&task_dir)?;

        // 读取status文件
        let status = read_status_file(&task_dir)?;

        // 构造响应
        Ok(AnsibleTaskStatusResponse {
            ident: input.ident,
            success: rc == 0,
            rc,
            status,
        })
    }

    // 辅助函数：读取rc文件
    fn read_rc_file(task_dir: &Path) -> Result<i32> {
        let rc_file = task_dir.join("rc");
        if !rc_file.exists() {
            return Err(anyhow::anyhow!("rc file not found at {}", rc_file.display()).into());
        }

        Ok(std::fs::read_to_string(&rc_file)
            .map_err(|e| anyhow::anyhow!("Failed to read rc file: {}", e))?
            .trim()
            .parse::<i32>()
            .map_err(|e| anyhow::anyhow!("Invalid rc value: {}", e))?)
    }

    // 辅助函数：读取status文件
    fn read_status_file(task_dir: &Path) -> Result<String> {
        let status_file = task_dir.join("status");
        if !status_file.exists() {
            return Err(
                anyhow::anyhow!("status file not found at {}", status_file.display()).into(),
            );
        }

        Ok(std::fs::read_to_string(&status_file)
            .map_err(|e| anyhow::anyhow!("Failed to read status file: {}", e))?)
    }

    // 辅助函数：构造未找到响应
    fn not_found_response(ident: &str, message: &str) -> Result<AnsibleTaskStatusResponse> {
        Ok(AnsibleTaskStatusResponse {
            ident: ident.to_string(),
            success: false,
            rc: 127,
            status: format!("ERROR: {message}"),
        })
    }

    // 包装器，将具体类型的函数适配到通用JSON接口
    struct JsonFunctionWrapper<F, I, O>
    where
        F: Fn(I) -> Result<O> + Send + Sync,
        I: for<'de> Deserialize<'de>,
        O: Serialize,
    {
        handler: F,
        _phantom_i: std::marker::PhantomData<I>,
        _phantom_o: std::marker::PhantomData<O>,
    }

    impl<F, I, O> JsonFunctionWrapper<F, I, O>
    where
        F: Fn(I) -> Result<O> + Send + Sync,
        I: for<'de> Deserialize<'de>,
        O: Serialize,
    {
        fn new(handler: F) -> Self {
            Self {
                handler,
                _phantom_i: std::marker::PhantomData,
                _phantom_o: std::marker::PhantomData,
            }
        }
    }

    impl<F, I, O> FunctionHandler<serde_json::Value, serde_json::Value> for JsonFunctionWrapper<F, I, O>
    where
        F: Fn(I) -> Result<O> + Send + Sync,
        I: for<'de> Deserialize<'de> + Send + Sync + 'static,
        O: Serialize + Send + Sync + 'static,
    {
        fn handle(&self, input: serde_json::Value) -> Result<serde_json::Value> {
            let concrete_input: I = serde_json::from_value(input)?;
            let output = (self.handler)(concrete_input)?;
            Ok(serde_json::to_value(output)?)
        }
    }
}
