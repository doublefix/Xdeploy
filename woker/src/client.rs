use super::*;
use function_handlers::FUNCTION_REGISTRY;
use prost::Message;
use std::collections::HashMap;
use std::time::SystemTime;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

pub mod agent {
    tonic::include_proto!("api");
}

use agent::{
    AgentMessage, CancelTask, FunctionRequest, FunctionResult, Heartbeat, TunnelMessage,
    TunnelPayload, TunnelResponse, agent_message::Body, agent_service_client::AgentServiceClient,
};

// 错误类型
type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

// 配置常量
const RECONNECT_INTERVAL: Duration = Duration::from_secs(5);
const MAX_RECONNECT_ATTEMPTS: usize = 1; // 重试 10 次

pub struct AgentClient {
    endpoint: String,
    agent_id: String,
}

impl AgentClient {
    pub fn new(endpoint: &str, agent_id: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            agent_id: agent_id.to_string(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut attempt = 0;

        loop {
            attempt += 1;
            println!("Attempting to connect (attempt {attempt}/{MAX_RECONNECT_ATTEMPTS})...");

            match self.try_connect().await {
                Ok(_) => {
                    println!("Connection to server lost, will attempt to reconnect...");
                    attempt = 0; // Reset attempt counter after successful connection
                }
                Err(e) => {
                    println!("Connection attempt failed: {e}");
                    if attempt >= MAX_RECONNECT_ATTEMPTS {
                        return Err("Max reconnect attempts reached".into());
                    }
                }
            }

            tokio::time::sleep(RECONNECT_INTERVAL).await;
        }
    }

    async fn try_connect(&self) -> Result<()> {
        let mut client = AgentServiceClient::connect(self.endpoint.clone()).await?;
        println!("Connected to server at {}", self.endpoint);

        let (tx, rx) = mpsc::channel(32);
        let outbound = ReceiverStream::new(rx);

        // 发送初始心跳
        self.send_initial_heartbeat(tx.clone()).await?;

        // 连接服务器流
        let mut stream = client
            .agent_stream(Request::new(outbound))
            .await?
            .into_inner();

        // 启动心跳任务
        let heartbeat_handle = self.spawn_heartbeat_task(tx.clone());

        // 处理服务器消息
        let result = self.handle_server_messages(&mut stream, tx).await;

        // 取消心跳任务
        heartbeat_handle.abort();

        result
    }

    async fn send_initial_heartbeat(&self, tx: mpsc::Sender<AgentMessage>) -> Result<()> {
        let initial_heartbeat = self.create_heartbeat();
        tx.send(initial_heartbeat).await?;
        Ok(())
    }

    fn spawn_heartbeat_task(&self, tx: mpsc::Sender<AgentMessage>) -> tokio::task::JoinHandle<()> {
        let agent_id = self.agent_id.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                let heartbeat = AgentMessage {
                    body: Some(Body::Heartbeat(Heartbeat {
                        agent_id: agent_id.clone(),
                        agent_type: "xdeployer".into(),
                        timestamp: current_timestamp(),
                    })),
                };
                if tx.send(heartbeat).await.is_err() {
                    println!("Heartbeat send failed, stream likely closed");
                    break;
                }
            }
        })
    }

    async fn handle_server_messages(
        &self,
        stream: &mut tonic::Streaming<AgentMessage>,
        tx: mpsc::Sender<AgentMessage>,
    ) -> Result<()> {
        while let Some(msg) = stream.message().await? {
            match msg.body {
                Some(Body::FunctionRequest(req)) => {
                    println!("Received FunctionRequest: {}", req.function_name);
                    self.handle_function_request(req, tx.clone()).await;
                }
                Some(Body::CancelTask(CancelTask { request_id })) => {
                    println!("Received CancelTask for request_id: {request_id}");
                    // TODO: 实现取消逻辑
                }
                Some(Body::Heartbeat(hb)) => {
                    println!("Received Heartbeat from server: {}", hb.agent_id);
                }
                Some(Body::TunnelMessage(tunnel_msg)) => {
                    println!(
                        "Received TunnelMessage with session_id: {}",
                        tunnel_msg.session_id
                    );
                    self.handle_tunnel_message(tunnel_msg, tx.clone()).await;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_tunnel_message(&self, msg: TunnelMessage, tx: mpsc::Sender<AgentMessage>) {
        tokio::spawn(async move {
            println!("Received TunnelMessage:");
            println!("Session ID: {}", msg.session_id);
            println!("Metadata: {:?}", msg.metadata);

            // 处理原始payload并准备响应字段
            let mut response_fields = HashMap::new();

            // 添加固定字段
            response_fields.insert(
                "processed_by".to_string(),
                json_to_prost_value(&serde_json::Value::String("rust-agent".to_string())),
            );
            response_fields.insert(
                "original_session".to_string(),
                json_to_prost_value(&serde_json::Value::String(msg.session_id.clone())),
            );

            // 处理并添加原始payload内容
            if let Some(payload) = msg.payload {
                match Self::unpack_tunnel_payload(payload) {
                    Ok(unpacked) => {
                        println!("TunnelPayload content:");
                        for (key, value) in unpacked.fields {
                            println!("{}: {:?}", key, prost_value_to_json(&value));
                            // 将原始payload的每个字段添加到响应中
                            response_fields.insert(format!("original.{key}"), value);
                        }
                    }
                    Err(e) => {
                        println!("Failed to unpack payload: {e}");
                        // 即使解包失败，也添加错误信息
                        response_fields.insert(
                            "payload_error".to_string(),
                            json_to_prost_value(&serde_json::Value::String(format!(
                                "Failed to unpack: {e}"
                            ))),
                        );
                    }
                }
            } else {
                println!("No payload in TunnelMessage");
            }

            // 创建响应payload
            let response_payload = TunnelPayload {
                fields: response_fields,
            };

            // 打包并发送响应
            let any_payload = prost_types::Any {
                type_url: "type.googleapis.com/api.TunnelPayload".to_string(),
                value: response_payload.encode_to_vec(),
            };

            let response = TunnelResponse {
                session_id: msg.session_id,
                random_number: rand::random::<i32>(),
                payload: Some(any_payload),
                status: "processed".to_string(),
            };

            if tx
                .send(AgentMessage {
                    body: Some(Body::TunnelResponse(response)),
                })
                .await
                .is_err()
            {
                eprintln!("Failed to send TunnelResponse");
            }
        });
    }

    fn unpack_tunnel_payload(
        any: prost_types::Any,
    ) -> std::result::Result<TunnelPayload, Box<dyn std::error::Error>> {
        if any.type_url != "type.googleapis.com/api.TunnelPayload" {
            return Err("Invalid type URL".into());
        }

        let payload = TunnelPayload::decode(&*any.value)?;
        Ok(payload)
    }

    async fn handle_function_request(&self, req: FunctionRequest, tx: mpsc::Sender<AgentMessage>) {
        tokio::spawn(async move {
            let response = handle_function(req).await;
            let msg = AgentMessage {
                body: Some(Body::FunctionResult(response)),
            };
            if tx.send(msg).await.is_err() {
                eprintln!("Failed to send FunctionResult");
            }
        });
    }

    fn create_heartbeat(&self) -> AgentMessage {
        AgentMessage {
            body: Some(Body::Heartbeat(Heartbeat {
                agent_id: self.agent_id.clone(),
                agent_type: "xdeployer".into(),
                timestamp: current_timestamp(),
            })),
        }
    }
}

async fn handle_function(req: FunctionRequest) -> FunctionResult {
    let json_value = req
        .parameters
        .as_ref()
        .map(struct_to_value)
        .unwrap_or_default();

    match FUNCTION_REGISTRY.get(&req.function_name) {
        Some(handler) => match handler.handle(json_value) {
            Ok(val) => FunctionResult {
                request_id: req.request_id,
                success: true,
                result: Some(value_to_struct(&val)),
                error_message: "".to_string(),
            },
            Err(e) => FunctionResult {
                request_id: req.request_id,
                success: false,
                result: None,
                error_message: e.to_string(),
            },
        },
        None => FunctionResult {
            request_id: req.request_id,
            success: false,
            result: None,
            error_message: "Unknown function".to_string(),
        },
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub mod ansible {
    tonic::include_proto!("ansible");
}

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

    use super::{
        types::{HelloInput, HelloOutput},
        *,
    };
    // use crate::types::{HelloInput, HelloOutput};

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

// 协议转换模块
use prost_types::{Struct, Value};
// Struct → serde_json::Value
pub fn struct_to_value(s: &Struct) -> serde_json::Value {
    let map = s
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    serde_json::Value::Object(map)
}

// serde_json::Value → Struct
pub fn value_to_struct(v: &serde_json::Value) -> Struct {
    if let serde_json::Value::Object(map) = v {
        let fields = map
            .iter()
            .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
            .collect();
        Struct { fields }
    } else {
        Struct {
            fields: Default::default(),
        }
    }
}

// prost_types::Value → serde_json::Value
pub fn prost_value_to_json(v: &Value) -> serde_json::Value {
    match &v.kind {
        Some(prost_types::value::Kind::NullValue(_)) => serde_json::Value::Null,
        Some(prost_types::value::Kind::NumberValue(n)) => serde_json::Value::from(*n),
        Some(prost_types::value::Kind::StringValue(s)) => serde_json::Value::from(s.clone()),
        Some(prost_types::value::Kind::BoolValue(b)) => serde_json::Value::from(*b),
        Some(prost_types::value::Kind::StructValue(s)) => struct_to_value(s),
        Some(prost_types::value::Kind::ListValue(l)) => {
            serde_json::Value::Array(l.values.iter().map(prost_value_to_json).collect())
        }
        None => serde_json::Value::Null,
    }
}

// serde_json::Value → prost_types::Value
pub fn json_to_prost_value(v: &serde_json::Value) -> Value {
    Value {
        kind: Some(match v {
            serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
            serde_json::Value::Bool(b) => prost_types::value::Kind::BoolValue(*b),
            serde_json::Value::Number(n) => {
                prost_types::value::Kind::NumberValue(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => prost_types::value::Kind::StringValue(s.clone()),
            serde_json::Value::Array(arr) => {
                prost_types::value::Kind::ListValue(prost_types::ListValue {
                    values: arr.iter().map(json_to_prost_value).collect(),
                })
            }
            serde_json::Value::Object(map) => prost_types::value::Kind::StructValue(Struct {
                fields: map
                    .iter()
                    .map(|(k, v)| (k.clone(), json_to_prost_value(v)))
                    .collect(),
            }),
        }),
    }
}
