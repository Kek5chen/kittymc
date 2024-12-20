use std::net::SocketAddr;
use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

fn modify_client_data(_data: &mut Vec<u8>, _n: usize) {}

fn modify_server_data(_data: &mut Vec<u8>, _n: usize) {}

async fn forward_data(
    mut reader: TcpStream,
    mut writer: TcpStream,
    is_client_to_server: bool,
) -> anyhow::Result<()> {
    let mut buffer = vec![0u8; 2048];

    loop {
        let n = match reader.read(&mut buffer).await {
            Ok(0) => {
                // The other side closed the connection
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => return Err(e.into()),
        };

        if is_client_to_server {
            modify_client_data(&mut buffer, n)
        } else {
            modify_server_data(&mut buffer, n)
        };

        writer.write_all(&buffer[..n]).await?;
    }
}

fn get_server_address() -> String {
    "gommehd.net".to_string()
}

async fn client_loop(client: TcpStream, sockaddr: &SocketAddr) -> anyhow::Result<()>{
    let server_url = get_server_address();
    let server_addr = (server_url.as_str(), 25565);
    let server = TcpStream::connect(server_addr)
        .await
        .with_context(|| format!("Failed to connect to server at {:?}", server_addr))?;

    println!("Established proxy between client {} and server {}", sockaddr, server_url);

    // ugly tokio::TcpStream::try_clone() https://github.com/tokio-rs/tokio-core/issues/198
    let std_client = client.into_std()?;
    let std_server = server.into_std()?;
    let client_for_server = TcpStream::from_std(std_client.try_clone()?)?;
    let server_for_client = TcpStream::from_std(std_server.try_clone()?)?;
    let client = TcpStream::from_std(std_client)?;
    let server = TcpStream::from_std(std_server)?;

    let client_to_server_task = tokio::spawn(async move {
        if let Err(e) = forward_data(client_for_server, server, true).await {
            eprintln!("Error forwarding client->server: {e}");
        }
    });

    let server_to_client_task = tokio::spawn(async move {
        if let Err(e) = forward_data(server_for_client, client, false).await {
            eprintln!("Error forwarding client->server: {e}");
        }
    });

    tokio::select! {
        _ = client_to_server_task => { Ok(()) }
        _ = server_to_client_task => { Ok(()) }
    }
}

async fn new_client_thread(client: TcpStream, sockaddr: SocketAddr) {
    match client_loop(client, &sockaddr).await {
        Err(e) => eprintln!("Fatal error in client {sockaddr}: {e}"),
        Ok(()) => println!("Client {sockaddr} disconnected."),
    }
}

async fn handle_new_client(server: &TcpListener) -> JoinHandle<()> {
    let (client, sockaddr) = server.accept().await.expect("Failed to accept");
    println!("Client {sockaddr} connected");
    tokio::spawn(async move {
        new_client_thread(client, sockaddr).await;
    })
}

#[tokio::main]
async fn main() {
    let server = TcpListener::bind(("0.0.0.0", 25565)).await.expect("Failed to bind");
    loop {
        handle_new_client(&server).await;
    }
}