use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, HttpResponse, Responder};
use actix_web::rt::spawn;
use actix_ws::{Message, Session, handle};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::env;
use log::error;

// Request
#[derive(Deserialize, Serialize, Debug, Clone)]
struct ActionPayload {
    message: String,
    room_name: String,
    session_id: String
}

//Response
// #[derive(Serialize)]
// struct SessionResponse {
//     session_id: String
// }


struct AppState {
    rooms: Arc<Mutex<HashMap<String, Vec<String>>>>,
    client_rooms: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    sessions: Mutex<HashMap<String, Session>>
}
impl AppState {
    // Subscribe To Room
    async fn subscribe_to_room(&self, session_id: String, room_name: &str, mut session: Session) {
        // Add client to a room
        let mut rooms = self.rooms.lock().unwrap();
        rooms.entry(room_name.to_string())
            .or_insert_with(Vec::new)
            .push(session_id.clone());
        session
            .text(format!("Add client to room: {}", room_name))
            .await
            .unwrap();
        // Save room for client
        let mut client_rooms = self.client_rooms.lock().unwrap();
        client_rooms
            .entry(session_id.clone())
            .or_insert_with(HashSet::new)
            .insert(room_name.to_string());
        session
            .text(format!("Add room {} to list of client {} room", room_name, session_id))
            .await
            .unwrap();
    }

    // Broadcast Message To A Room
    async fn broadcast_to_room(&self, room_name: &str, message: &str, mut session: Session) {
        println!("Broadcasting text to {room_name}: {message}");
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(session_ids) = rooms.get_mut(room_name) {
            let mut sessions = self.sessions.lock().unwrap();

            for session_id in session_ids {
                if let Some(client_session) = sessions.get_mut(session_id) {
                    let _ = client_session.text(message.to_string()).await;
                }
            }
        } else {
            session
                .text(format!("Room {} does not exist.", room_name))
                .await
                .unwrap();
        }
    }

    // Unsubscribe From A Room
    async fn unsubscribe_from_room(&self, session_id: String, room_name: &str, mut session: Session) {
        // Remove client from a room
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(clients) = rooms.get_mut(room_name) {
            clients.retain(|s| *s != session_id);
            session
                .text(format!("Remove client from room: {}", room_name))
                .await
                .unwrap();
        }
        // Remove room from client
        let mut client_rooms = self.client_rooms.lock().unwrap();
        if let Some(rooms) = client_rooms.get_mut(&session_id) {
            rooms.remove(room_name);
            if rooms.is_empty() {
                client_rooms.remove(&session_id);
                session
                    .text(format!("Remove room {} from client {}", room_name, session_id))
                    .await
                    .unwrap();
            }
        }
    }

    // Get List Of Rooms
    async fn get_client_rooms(&self, session_id: String) -> Vec<String> {
        let client_rooms = self.client_rooms.lock().unwrap();
        client_rooms
            .get(&session_id)
            .map(|rooms| rooms.iter().cloned().collect())
            .unwrap_or_else(Vec::new)
    }
    
    // Get Session By ID
    fn get_session_by_id(&self, session_id: String) -> Option<Session> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(found_session) = sessions.get(&session_id) {
            Some(found_session.clone())
        } else {
            None
        }
    }
}

/** Endpoint Handler **/
// `/send` endpoint handler 
async fn send_message(
    payload: web::Json<ActionPayload>,
    data: web::Data<Arc<AppState>>
) -> impl Responder {
    let curr_session = data.get_session_by_id(payload.session_id.clone());

    match curr_session {
        Some(curr_session) => {
            data.broadcast_to_room(&payload.room_name, &payload.message, curr_session).await;
            HttpResponse::Ok().json("Message sent")
        }
        None => {
            HttpResponse::InternalServerError().json("Fail to get session")
        }
    }
}
// `/subscribe` endpoint handler 
async fn subscribe_room(
    payload: web::Json<ActionPayload>,
    data: web::Data<Arc<AppState>>
) -> impl Responder {
    let curr_session = data.get_session_by_id(payload.session_id.clone());
    
    match curr_session {
        Some(curr_session) => {
            data.subscribe_to_room(payload.session_id.clone(), &payload.room_name, curr_session).await;
            HttpResponse::Ok().json(format!("Subscribed to {}", payload.room_name))
        }
        None => {
            HttpResponse::InternalServerError().json("Fail to get session")
        }
    }
}
// `/unsubscribe` endpoint handler 
async fn unsubscribe_room(
    payload: web::Json<ActionPayload>,
    data: web::Data<Arc<AppState>>
) -> impl Responder {
    let curr_session = data.get_session_by_id(payload.session_id.clone());
    
    match curr_session {
        Some(curr_session) => {
            data.unsubscribe_from_room(payload.session_id.clone(), &payload.room_name, curr_session).await;
            HttpResponse::Ok().json(format!("Unsubscribed from {}", payload.room_name))
        }
        None => {
            HttpResponse::InternalServerError().json("Fail to get session")
        }
    }
}
// `/list` endpoint handler 
async fn get_all_room(
    payload: web::Json<ActionPayload>,
    data: web::Data<Arc<AppState>>
) -> impl Responder {
    let all_client_room = data.get_client_rooms(payload.session_id.clone()).await;
    HttpResponse::Ok().json(format!("{}", all_client_room.join(";")))
}


// Websocket initialization
async fn ws(
    req: HttpRequest,
    body: web::Payload,
    data: web::Data<Arc<AppState>>
) -> actix_web::Result<impl Responder> {
    // Handshake to websocket
    let (response, mut session, mut msg_stream) = handle(&req, body)?;

    let session_id = Uuid::new_v4().to_string();
    let state = data.get_ref().clone();
    {
        let mut sessions = state.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session.clone());
    }
    println!("New WebSocket session created with ID: {}", session_id);

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

                    let msg = msg.trim();

                    // Handle command
                    if msg.starts_with('/') {
                        let mut cmd_args = msg.splitn(2, ' ');

                        match cmd_args.next().unwrap() {
                            "/subscribe" => {
                                if let Some(room_name) = cmd_args.next() {
                                    state.subscribe_to_room(session_id.clone(), room_name, session.clone()).await;
                                    println!("Subcribed to room: {}", room_name);
                                } else {
                                    session
                                        .text("Please provide a room name to subscribe.")
                                        .await
                                        .unwrap();
                                }
                            }

                            "/unsubscribe" => {
                                if let Some(room_name) = cmd_args.next() {
                                    state.unsubscribe_from_room(session_id.clone(), room_name, session.clone()).await;
                                    println!("Unsubcribed to room: {}", room_name);
                                } else {
                                    session
                                        .text("Please provide a room name to unsubscribe.")
                                        .await
                                        .unwrap();
                                }
                            }

                            "/list" => {
                                println!("Listing subscription...");
                                session
                                    .text("Listing subscription...")
                                    .await
                                    .unwrap();
                                let rooms = state.get_client_rooms(session_id.clone()).await;
                                if rooms.is_empty() {
                                    session
                                        .text("No rooms found for this session.")
                                        .await
                                        .unwrap();
                                } else {
                                    for room in rooms {
                                        session
                                            .text(format!("{}", room))
                                            .await
                                            .unwrap();
                                    }
                                }
                            }

                            "/send" => {
                                if let Some(full_args) = cmd_args.next() {
                                    let mut cmd_msg = full_args.splitn(2, ' ');
                                    if let (Some(room_name), Some(message)) = (cmd_msg.next(), cmd_msg.next()) {
                                        let _ = (room_name, message);
                                        println!("Broadcasting to room {}: {}", room_name, message);
                                        state.broadcast_to_room(room_name, message, session.clone()).await;
                                    } else {
                                        session.text("Please provide a message to broadcast.").await.unwrap();
                                    }
                                } else {
                                    session.text("Please provide a room name to broadcast to.").await.unwrap();
                                }
                            }
                            
                            "/current" => {
                                let curr_session = session_id.clone();
                                session
                                    .text(format!("Your session ID: {}", curr_session))
                                    .await
                                    .unwrap();
                            }

                            _ => {
                                session
                                    .text(format!("!!! unknown command: {msg}"))
                                    .await
                                    .unwrap();
                            }
                        }
                    } else {
                        if session.text(msg).await.is_err() {
                            error!("Error sending message");
                        }
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

// Main
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let state = Arc::new(AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
        client_rooms: Arc::new(Mutex::new(HashMap::new())),
        sessions: Mutex::new(HashMap::new())
    });

    // Binding a Listener
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .route("/ws", web::get().to(ws))
            .route("/send", web::post().to(send_message))
            .route("/subscribe", web::post().to(subscribe_room))
            .route("/unsubscribe", web::post().to(unsubscribe_room))
            .route("/list", web::post().to(get_all_room))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}