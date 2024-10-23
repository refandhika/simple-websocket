use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures::{StreamExt, SinkExt};
use std::env;
use std::net::SocketAddr;
use log::{info, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Binding a Listener
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let addr: SocketAddr = addr.parse().expect("Invalid address");
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind listener");
    info!("Listening on: {}", addr);
    println!("Listening on: {}", addr);

    // Run handle_conn() for each listener connection
    while let Ok((stream, _ )) = listener.accept().await {
        tokio::spawn(handle_conn(stream));
    }
}


async fn handle_conn(stream: TcpStream) {
    // Accept connection as ws_stream
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during the websocket handshake: {}" , e);
            return;
        }
    };

    // Split stream to send and recieve
    let (mut send, mut receive) = ws_stream.split();

    // Handle message
    while let Some(msg) = receive.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Reversing message
                let reversed = text.chars().rev().collect::<String>();

                if let Err(e) = send.send(Message::Text(reversed)).await {
                    error!("Error sending message: {}", e);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => (),
            Err(e) => {
                error!("Error processing message: {}", e);
                break;
            }
        }
    }
}