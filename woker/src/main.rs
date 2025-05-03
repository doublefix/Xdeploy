use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hardcoded connection details
    let host = "localhost";
    let port = 22;
    let username = "xxx";
    let password = "xxx";

    // Connect to the SSH server
    let tcp = TcpStream::connect(format!("{host}:{port}"))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate
    sess.userauth_password(username, password)?;
    if !sess.authenticated() {
        return Err("Authentication failed".into());
    }

    // Open a channel and request a PTY (xterm)
    let mut channel = sess.channel_session()?;
    channel.request_pty("xterm", None, Some((80, 24, 0, 0)))?;
    channel.shell()?;

    println!("SSH session established. Type 'exit' to quit.");

    // Simple loop to read from stdin and write to channel
    let mut input = String::new();
    let mut buf = [0; 1024];

    loop {
        // Read from channel (server output)
        match channel.read(&mut buf) {
            Ok(n) if n > 0 => {
                print!("{}", String::from_utf8_lossy(&buf[..n]));
            }
            _ => {}
        }

        // Read from stdin (user input)
        std::io::stdin().read_line(&mut input)?;
        if input.trim() == "exit" {
            break;
        }

        // Write to channel
        channel.write_all(input.as_bytes())?;
        input.clear();
    }

    channel.close()?;
    println!("Session closed");
    Ok(())
}
