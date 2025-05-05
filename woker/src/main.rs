use once_cell::sync::Lazy;
use prost_types::{Struct, Value};
use serde::{Deserialize, Serialize};
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
    AgentMessage, CancelTask, FunctionRequest, FunctionResult, Heartbeat, agent_message::Body,
    agent_service_client::AgentServiceClient,
};

// Function handler 类型和注册表
type FunctionHandler =
    fn(&serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;

static FUNCTION_REGISTRY: Lazy<HashMap<String, FunctionHandler>> = Lazy::new(|| {
    let mut map: HashMap<String, FunctionHandler> = HashMap::new();
    map.insert("Hello".to_string(), hello_handler);
    map
});

#[derive(Debug, Serialize, Deserialize)]
struct InputStruct {
    name: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputStruct {
    greeting: String,
    original: InputStruct,
}

fn hello_handler(
    params: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let input: InputStruct = serde_json::from_value(params.clone())?;
    let output = OutputStruct {
        greeting: format!("Hello, {}", input.name),
        original: input,
    };
    Ok(serde_json::to_value(output)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting agent client...");
    let endpoint = "http://localhost:50051";
    let agent_id = "rust-agent-001";

    let mut client = AgentServiceClient::connect(endpoint.to_string()).await?;

    println!("Connected to server at {}", endpoint);
    let (tx, rx) = mpsc::channel(32);
    let outbound = ReceiverStream::new(rx);
    println!("Creating outbound stream");

    // 👇 立即发送一条消息，避免 server 端阻塞
    let initial_heartbeat = AgentMessage {
        body: Some(Body::Heartbeat(Heartbeat {
            agent_id: agent_id.to_string(),
            agent_type: "xdeployer".into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        })),
    };
    tx.send(initial_heartbeat).await?;

    // 再连接 stream
    let mut stream = client
        .agent_stream(Request::new(outbound))
        .await?
        .into_inner();

    println!("Connected to server stream");
    let tx_clone = tx.clone();
    let agent_id_clone = agent_id.to_string();

    // 心跳任务
    println!("Starting heartbeat task");
    tokio::spawn(async move {
        loop {
            let heartbeat = AgentMessage {
                body: Some(Body::Heartbeat(Heartbeat {
                    agent_id: agent_id_clone.clone(),
                    agent_type: "xdeployer".into(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                })),
            };
            if tx_clone.send(heartbeat).await.is_err() {
                println!("stream closed, heartbeat task exiting");
                break;
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    // 接收服务端消息
    while let Some(msg) = stream.message().await? {
        match msg.body {
            Some(Body::FunctionRequest(req)) => {
                println!("Received FunctionRequest: {}", req.function_name);
                let tx = tx.clone();
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
            Some(Body::CancelTask(CancelTask { request_id })) => {
                println!("Received CancelTask for request_id: {}", request_id);
                // TODO: 实现取消逻辑
            }
            Some(Body::Heartbeat(hb)) => {
                println!("Received Heartbeat from server: {}", hb.agent_id);
            }
            _ => {}
        }
    }

    println!("Server closed stream");
    Ok(())
}

// 处理函数调用
async fn handle_function(req: FunctionRequest) -> FunctionResult {
    let json_value = req
        .parameters
        .as_ref()
        .map(struct_to_value)
        .unwrap_or_default();

    if let Some(handler) = FUNCTION_REGISTRY.get(&req.function_name) {
        match handler(&json_value) {
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
        }
    } else {
        FunctionResult {
            request_id: req.request_id,
            success: false,
            result: None,
            error_message: "Unknown function".to_string(),
        }
    }
}

// Struct → serde_json::Value
fn struct_to_value(s: &Struct) -> serde_json::Value {
    let map = s
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), prost_value_to_json(v)))
        .collect::<serde_json::Map<_, _>>();
    serde_json::Value::Object(map)
}

// serde_json::Value → Struct
fn value_to_struct(v: &serde_json::Value) -> Struct {
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
fn prost_value_to_json(v: &Value) -> serde_json::Value {
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
fn json_to_prost_value(v: &serde_json::Value) -> Value {
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
