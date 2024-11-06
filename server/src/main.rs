#![feature(let_chains)]

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use crate::user::User;

mod event_listener;
mod room_manager;
mod user;
mod sfu;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    tokio::spawn(event_listener::listening_for_course_creations());

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake");

            let user = Arc::new(Mutex::new(User::new(ws_stream)));
            User::handle_ws_actions(user).await;
        });
    }
    Ok(())
}
