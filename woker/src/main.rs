use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_stream::StreamExt;
use tonic::{Request, transport::Channel};

pub mod ssh {
    tonic::include_proto!("ssh");
}

use ssh::{
    SshError, SshInit, SshInput, SshOutput, SshStream, ssh_service_client::SshServiceClient,
};

struct SshSession {
    tx: mpsc::Sender<SshInput>,
    #[allow(dead_code)]
    task: JoinHandle<()>,
}

#[derive(Clone)]
struct AppState {
    ssh_sessions: Arc<Mutex<HashMap<String, SshSession>>>,
    grpc_client: SshServiceClient<Channel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到gRPC服务
    let grpc_addr = "http://localhost:50052";
    let client = SshServiceClient::connect(grpc_addr).await?;

    let state = AppState {
        ssh_sessions: Arc::new(Mutex::new(HashMap::new())),
        grpc_client: client,
    };

    // 创建双向流
    let (_tx, rx) = mpsc::channel(100);
    let request_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let mut response_stream = state
        .grpc_client
        .clone()
        .start_stream(Request::new(request_stream))
        .await?
        .into_inner();

    // 处理来自服务器的消息
    while let Some(message) = response_stream.next().await {
        match message {
            Ok(stream) => handle_server_message(stream, state.clone()).await?,
            Err(e) => eprintln!("gRPC error: {e}"),
        }
    }

    Ok(())
}

async fn handle_server_message(
    stream: SshStream,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(payload) = stream.payload {
        println!("Received payload: {payload:?}");
        match payload {
            ssh::ssh_stream::Payload::Init(init) => {
                start_ssh_session(init, stream.session_id, state).await?
            }
            ssh::ssh_stream::Payload::Input(input) => {
                send_ssh_input(stream.session_id, input, state).await?
            }
            _ => {}
        }
    }
    Ok(())
}

async fn start_ssh_session(
    init: SshInit,
    session_id: String,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // 建立 SSH 连接
    let tcp = TcpStream::connect(format!("{}:{}", init.host, init.port))?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(&init.username, &init.password)?;
    let mut channel = sess.channel_session()?;
    channel.request_pty(
        "xterm",
        None,
        Some((init.cols as u32, init.rows as u32, 0, 0)),
    )?;
    channel.shell()?;

    // 创建 channel 通道用于异步写入
    let (tx, mut rx) = mpsc::channel::<SshInput>(100);

    // 启动任务读取 SSH 输出
    let mut read_channel = channel.clone(); // ssh2::Channel 不实现 Send，所以用 clone
    let client_clone = state.grpc_client.clone();
    let session_id_clone = session_id.clone();

    let output_task = tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            match read_channel.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let output = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = client_clone
                        .clone()
                        .start_stream(Request::new(tokio_stream::iter(vec![SshStream {
                            session_id: session_id_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Output(SshOutput {
                                data: output,
                            })),
                        }])))
                        .await;
                }
                Ok(_) => {
                    // EOF
                    break;
                }
                Err(e) => {
                    let _ = client_clone
                        .clone()
                        .start_stream(Request::new(tokio_stream::iter(vec![SshStream {
                            session_id: session_id_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Error(SshError {
                                message: e.to_string(),
                            })),
                        }])))
                        .await;
                    break;
                }
            }
        }
    });

    // 启动任务读取输入并写入 SSH
    let mut write_channel = channel;
    tokio::spawn(async move {
        while let Some(input) = rx.recv().await {
            if let Err(e) = write_channel.write_all(input.data.as_bytes()) {
                eprintln!("Failed to write to SSH: {e}");
                break;
            }
        }
    });

    // 保存 session
    state.ssh_sessions.lock().await.insert(
        session_id,
        SshSession {
            tx,
            task: output_task,
        },
    );

    Ok(())
}

async fn send_ssh_input(
    session_id: String,
    input: SshInput,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(session) = state.ssh_sessions.lock().await.get(&session_id) {
        session.tx.send(input).await?;
    }
    Ok(())
}
