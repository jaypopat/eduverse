use crate::user::User;
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
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    // listen for course creation smart contract event and create room if teacher creates a course
    tokio::spawn(event_listener::listening_for_course_creations());

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake");

            // creates a new user by assigning the websocket to url and then handles the user's actions
            let user = Arc::new(Mutex::new(User::new(ws_stream)));
            User::handle_ws_actions(user).await;
        });
    }
    Ok(())
}
