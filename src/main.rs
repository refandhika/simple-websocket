use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, HttpResponse, Responder};
use actix_web::rt::spawn;
use actix_ws::{Message, Session, handle};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::env;
use log::error;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct MessagePayload {
    message: String
}


struct AppState {
    sessions: Mutex<Vec<Session>>
}
impl AppState {
    // Broadcast Message To All
    async fn broadcast_message(&self, message: &str) {
        println!("Broadcasting text: {message}");
        let mut sessions = self.sessions.lock().unwrap();
        for session in sessions.iter_mut() {
            let _ = session.text(message).await;
        }
    }
}

// `/send` endpoint handler 
async fn send_message(
    payload: web::Json<MessagePayload>,
    data: web::Data<Arc<AppState>>
) -> impl Responder {
    data.broadcast_message(&payload.message).await;
    HttpResponse::Ok().json("Message sent")
}


// Websocket initialization
async fn ws(
    req: HttpRequest,
    body: web::Payload,
    data: web::Data<Arc<AppState>>
) -> actix_web::Result<impl Responder> {
    // Handshake to websocket
    let (response, mut session, mut msg_stream) = handle(&req, body)?;

    let state = data.get_ref().clone();
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.push(session.clone());
    }

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

                    // Return the reverse only to the sending user
                    println!("Sending reversed text: {reversed}");
                    if session.text(reversed).await.is_err() {
                        error!("Error sending message");
                    }
                },
                Message::Close(_) => break,
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let state = Arc::new(AppState {
        sessions: Mutex::new(Vec::new())
    });

    // Binding a Listener
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .route("/ws", web::get().to(ws))
            .route("/send", web::post().to(send_message))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}