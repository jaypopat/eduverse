use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("WebSocket server listening on: {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");
            let (mut write, mut read) = ws_stream.split();
            while let Some(message) = read.next().await {
                println!("{:?}", message);
                let message = message.expect("Error reading message");
                // println!("received message,{:?}", message);
                let message_to_send = Message::Text("hey i got your message ".to_string() + message.to_text().unwrap());
                write.send(message_to_send).await.expect("Error sending message");
            }
        });
    }
}