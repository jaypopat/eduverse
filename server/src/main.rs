use crate::room_manager::RoomManager;
use crate::user::User;
use event_listener::listening_for_course_creations;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

mod event_listener;
mod room_manager;
mod stream_types;
mod user;
mod ws_payload;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the RoomManager
    RoomManager::instance().initialize().await?;

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    // Spawn the event listener as a separate task
    let event_listener_handle = tokio::spawn(listening_for_course_creations());

    // Handle WebSocket connections
    let server_handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        println!("got a new client connection");
                        let user = Arc::new(Mutex::new(User::new(ws_stream)));
                        User::handle_ws_actions(user).await
                    }
                    Err(e) => eprintln!("Error during the WebSocket handshake: {:?}", e),
                }
            });
        }
    });

    tokio::select! {
        _ = server_handle => println!("WebSocket server task completed"),
        _ = event_listener_handle => println!("Event listener task completed"),
    }

    Ok(())
}
