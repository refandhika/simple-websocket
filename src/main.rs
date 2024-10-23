use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_web::rt::spawn;
use actix_ws::{Message, handle};
use futures::StreamExt;
use std::env;
use log::error;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    // Handshake to websocket
    let (response, mut session, mut msg_stream) = handle(&req, body)?;

    // Spawn handler
    spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => {
                    println!("Got text: {msg}");

                    // Reversing message
                    let reversed = msg.chars().rev().collect::<String>();

                    println!("Sending reversed text: {reversed}");
                    if session.text(reversed).await.is_err() {
                        error!("Error sending message");
                    }
                },
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Binding a Listener
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    println!("Listening on: {}", addr);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/ws", web::get().to(ws))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}