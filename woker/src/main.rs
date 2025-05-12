mod deploy;

use prost_types::{Struct, Value};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;

pub mod agent {
    tonic::include_proto!("api"); // proto package 名为 agent
}

use agent::{
    AgentMessage, CancelTask, FunctionRequest, FunctionResult, Heartbeat, TunnelMessage,
    TunnelPayload, TunnelResponse, agent_message::Body, agent_service_client::AgentServiceClient,
};

pub mod ansible {
    tonic::include_proto!("ansible");
}

// 错误类型
type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

// 配置常量
const RECONNECT_INTERVAL: Duration = Duration::from_secs(5);
const MAX_RECONNECT_ATTEMPTS: usize = 10;

// 共享类型

// 协议转换模块
mod proto_convert {
    use super::*;

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
}

// 客户端模块
mod client {
    use super::*;
    use prost::Message;
    use proto_convert::{struct_to_value, value_to_struct};
    use woker::function_handlers::FUNCTION_REGISTRY;

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

        fn spawn_heartbeat_task(
            &self,
            tx: mpsc::Sender<AgentMessage>,
        ) -> tokio::task::JoinHandle<()> {
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
                    proto_convert::json_to_prost_value(&serde_json::Value::String(
                        "rust-agent".to_string(),
                    )),
                );
                response_fields.insert(
                    "original_session".to_string(),
                    proto_convert::json_to_prost_value(&serde_json::Value::String(
                        msg.session_id.clone(),
                    )),
                );

                // 处理并添加原始payload内容
                if let Some(payload) = msg.payload {
                    match Self::unpack_tunnel_payload(payload) {
                        Ok(unpacked) => {
                            println!("TunnelPayload content:");
                            for (key, value) in unpacked.fields {
                                println!(
                                    "{}: {:?}",
                                    key,
                                    proto_convert::prost_value_to_json(&value)
                                );
                                // 将原始payload的每个字段添加到响应中
                                response_fields.insert(format!("original.{key}"), value);
                            }
                        }
                        Err(e) => {
                            println!("Failed to unpack payload: {e}");
                            // 即使解包失败，也添加错误信息
                            response_fields.insert(
                                "payload_error".to_string(),
                                proto_convert::json_to_prost_value(&serde_json::Value::String(
                                    format!("Failed to unpack: {e}"),
                                )),
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

        async fn handle_function_request(
            &self,
            req: FunctionRequest,
            tx: mpsc::Sender<AgentMessage>,
        ) {
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
}

// 辅助函数
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting agent client...");
    let endpoint = "http://localhost:50051";
    let agent_id = "rust-agent-001";

    let client = client::AgentClient::new(endpoint, agent_id);
    client.run().await?;

    println!("Server closed stream");
    Ok(())
}

// websocat ws://localhost:8080/api/v1/tunnel/rust-agent-001
// {"payload":{"input":"pwd"},"metadata":{"source":"test"}}
