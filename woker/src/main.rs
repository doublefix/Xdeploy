use futures::{Stream, StreamExt};
use ssh2::{Channel, Session};
use std::{
    io::{Read, Write},
    net::TcpStream,
    pin::Pin,
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

pub mod ssh {
    tonic::include_proto!("ssh");
}

use ssh::{
    SshEnd, SshError, SshInit, SshOutput, SshStream,
    ssh_service_server::{SshService, SshServiceServer},
};

struct SshServiceImpl;

#[tonic::async_trait]
impl SshService for SshServiceImpl {
    type StartStreamStream = Pin<Box<dyn Stream<Item = Result<SshStream, Status>> + Send>>;

    async fn start_stream(
        &self,
        request: Request<Streaming<SshStream>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        let mut input_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            if let Some(init_msg) = input_stream.next().await {
                match init_msg {
                    Ok(stream) => {
                        if let Some(ssh::ssh_stream::Payload::Init(init)) = stream.payload {
                            match handle_ssh_session(init, input_stream, tx.clone()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    let _ = tx.send(Err(e)).await;
                                }
                            }
                        } else {
                            let _ = tx
                                .send(Err(Status::invalid_argument(
                                    "First message must be of type Init",
                                )))
                                .await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

async fn handle_ssh_session(
    init: SshInit,
    mut input_stream: Streaming<SshStream>,
    tx: mpsc::Sender<Result<SshStream, Status>>,
) -> Result<(), Status> {
    let session_id = Uuid::new_v4().to_string();

    let tcp_stream = TcpStream::connect(format!("{}:{}", init.host, init.port))
        .map_err(|e| Status::internal(format!("Failed to connect to SSH: {e}")))?;

    let mut channel = start_ssh_session(
        tcp_stream,
        &init.username,
        &init.password,
        init.rows,
        init.cols,
    )
    .map_err(|e| Status::internal(format!("SSH session initialization failed: {e}")))?;

    let _ = tx
        .send(Ok(SshStream {
            session_id: session_id.clone(),
            payload: Some(ssh::ssh_stream::Payload::Init(init)),
        }))
        .await;

    let tx_read = tx.clone();
    let channel_clone = channel.stream(0);
    let sid_clone = session_id.clone();

    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match tokio::task::spawn_blocking({
                let mut ch = channel_clone.clone();
                move || ch.read(&mut buf)
            })
            .await
            {
                Ok(Ok(n)) if n > 0 => {
                    let output = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx_read
                        .send(Ok(SshStream {
                            session_id: sid_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Output(SshOutput {
                                data: output,
                            })),
                        }))
                        .await;
                }
                Ok(Ok(_)) => break,
                Ok(Err(e)) => {
                    let _ = tx_read
                        .send(Ok(SshStream {
                            session_id: sid_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Error(SshError {
                                message: format!("Read failed: {e}"),
                            })),
                        }))
                        .await;
                    break;
                }
                Err(e) => {
                    let _ = tx_read
                        .send(Ok(SshStream {
                            session_id: sid_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Error(SshError {
                                message: format!("Task failed: {e}"),
                            })),
                        }))
                        .await;
                    break;
                }
            }
        }
    });

    while let Some(input) = input_stream.next().await {
        match input {
            Ok(stream) => {
                if let Some(ssh::ssh_stream::Payload::Input(input)) = stream.payload {
                    if let Err(e) = channel.write_all(input.data.as_bytes()) {
                        let _ = tx
                            .send(Ok(SshStream {
                                session_id: session_id.clone(),
                                payload: Some(ssh::ssh_stream::Payload::Error(SshError {
                                    message: format!("Write failed: {e}"),
                                })),
                            }))
                            .await;
                        break;
                    }
                }
            }
            Err(e) => return Err(e),
        }
    }

    let exit_code = channel.exit_status().unwrap_or(-1);
    let _ = tx
        .send(Ok(SshStream {
            session_id,
            payload: Some(ssh::ssh_stream::Payload::End(SshEnd { exit_code })),
        }))
        .await;

    Ok(())
}

fn start_ssh_session(
    stream: TcpStream,
    username: &str,
    password: &str,
    rows: i32,
    cols: i32,
) -> Result<Channel, ssh2::Error> {
    let mut sess = Session::new()?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    sess.userauth_password(username, password)?;

    let mut channel = sess.channel_session()?;
    channel.request_pty("xterm", None, Some((cols as u32, rows as u32, 0, 0)))?;
    channel.shell()?;

    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let service = SshServiceImpl;

    println!("Starting SSH service on {addr}");
    tonic::transport::Server::builder()
        .add_service(SshServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
