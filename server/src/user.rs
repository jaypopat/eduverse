use std::sync::Arc;

use crate::room_manager::RoomManager;
use futures_util::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, ByteArray, Pair};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

pub struct User {
    pub(crate) id: Option<String>,
    room_id: Option<u32>,
    coordinates: (i32, i32),
    pub websocket: WebSocketStream<TcpStream>,
}
#[derive(Serialize, Deserialize)]
struct MovementPayload {
    x: i32,
    y: i32,
}
#[derive(Deserialize)]
struct JoinPayload {
    course_id: u32,
    pub_address: String,
    signature: String,
    message_signed: String,
}
#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
enum UserAction {
    #[serde(rename = "join")]
    JoinRoom(JoinPayload),
    #[serde(rename = "leave")]
    LeaveRoom,
    #[serde(rename = "move")]
    MoveTo(MovementPayload),
    #[serde(rename = "send_message")]
    SendMessage(String),
}
impl User {
    pub fn new(websocket: WebSocketStream<TcpStream>) -> Self {
        User {
            id: None,
            room_id: None,
            coordinates: (0, 0),
            websocket,
        }
    }
    pub async fn handle_ws_actions(user_arc: Arc<Mutex<Self>>) {
        loop {
            let message = {
                let mut user = user_arc.lock().await;
                user.websocket.next().await
            };

            match message {
                Some(Ok(msg)) => {
                    if let Ok(message_str) = msg.to_text() {
                        match serde_json::from_str::<UserAction>(message_str) {
                            Ok(action) => match action {
                                UserAction::JoinRoom(payload) => {
                                    Self::join_room(user_arc.clone(), payload).await;
                                }
                                UserAction::LeaveRoom => {
                                    Self::handle_leave_room(user_arc.clone()).await;
                                }
                                UserAction::MoveTo(coordinates) => {
                                    Self::handle_move_to(user_arc.clone(), coordinates).await;
                                }
                                UserAction::SendMessage(message) => {
                                    Self::handle_send_message(user_arc.clone(), message).await;
                                }
                            },
                            Err(e) => println!("Failed to parse message: {}", e),
                        }
                    }
                }
                Some(Err(e)) => println!("Error receiving message: {}", e),
                None => break,
            }
        }
    }
    async fn join_room(user_arc: Arc<Mutex<Self>>, payload: JoinPayload) {
        if !RoomManager::instance()
            .rooms
            .lock()
            .await
            .contains_key(&payload.course_id)
        {
            return;
        }

        let (course_id, pub_address, signature, message_signed) = (
            payload.course_id,
            payload.pub_address,
            payload.signature,
            payload.message_signed,
        );

        // Verify the signature
        if let Err(e) = verify_signature(pub_address.clone(), signature, message_signed) {
            eprintln!("Signature verification failed: {}", e);
            return;
        }

        let mut user = user_arc.lock().await;
        if user.id.is_none() {
            user.id = Some(pub_address.clone());
        }
        user.room_id = Some(course_id);
        user.coordinates = get_rand_coordinates();

        RoomManager::instance()
            .add_user_to_room(payload.course_id, user_arc.clone())
            .await;

        let join_message = serde_json::json!({
            "type": "user_joined",
            "user_id": user.id,
            "coordinates": user.coordinates
        });

        RoomManager::instance()
            .broadcast_message(user.id.clone(), course_id, join_message.to_string())
            .await;
    }
    async fn handle_leave_room(user_arc: Arc<Mutex<Self>>) {
        let (room_id, user_id) = {
            let user = user_arc.lock().await;
            (user.room_id, user.id.clone())
        };

        if let (Some(room_id), Some(user_id)) = (room_id, user_id) {
            RoomManager::instance()
                .remove_user_from_room(room_id, user_id.clone())
                .await;

            let leave_message = serde_json::json!({
                "type": "user_left",
                "user_id": user_id.clone()
            });

            RoomManager::instance()
                .broadcast_message(Some(user_id), room_id, leave_message.to_string())
                .await;

            user_arc.lock().await.room_id = None;
        }
    }

    async fn handle_move_to(user_arc: Arc<Mutex<Self>>, coordinates: MovementPayload) {
        let (room_id, user_id, valid_move) = {
            let mut user = user_arc.lock().await;
            let x_displacement = (user.coordinates.0 - coordinates.x).abs();
            let y_displacement = (user.coordinates.1 - coordinates.y).abs();
            let valid_move = (x_displacement == 1 && y_displacement == 0)
                || (x_displacement == 0 && y_displacement == 1);

            if valid_move {
                user.coordinates = (coordinates.x, coordinates.y);
            }

            (user.room_id, user.id.clone(), valid_move)
        };

        if let (Some(room_id), Some(user_id)) = (room_id, user_id) {
            if valid_move {
                let move_message = serde_json::json!({
                    "type": "user_moved",
                    "user_id": user_id,
                    "coordinates": coordinates
                });

                RoomManager::instance()
                    .broadcast_message(Some(user_id), room_id, move_message.to_string())
                    .await;
            } else {
                let reject_message = serde_json::json!({
                    "type": "movement_rejected",
                    "user_id": user_id,
                    "coordinates": coordinates
                });

                RoomManager::instance()
                    .broadcast_message(Some(user_id), room_id, reject_message.to_string())
                    .await;
            }
        }
    }

    async fn handle_send_message(user_arc: Arc<Mutex<Self>>, message: String) {
        let (room_id, user_id) = {
            let user = user_arc.lock().await;
            (user.room_id, user.id.clone())
        };

        if let (Some(room_id), Some(user_id)) = (room_id, user_id) {
            let message_payload = serde_json::json!({
                "type": "message",
                "sender": user_id.clone(),
                "content": message
            });

            RoomManager::instance()
                .broadcast_message(Some(user_id), room_id, message_payload.to_string())
                .await;
        }
    }
}

fn verify_signature(
    pub_address: String,
    signature: String,
    message_signed: String,
) -> Result<(), &'static str> {
    // Signature in bytes
    let signature_bytes = hex::decode(signature).expect("Invalid hex for signature");
    let signature = sr25519::Signature::from_slice(&signature_bytes).expect("Invalid signature");

    // Public key in bytes
    let public_key_bytes = hex::decode(pub_address).expect("Invalid hex for public key");
    let public_key = sr25519::Public::from_slice(&public_key_bytes).expect("Invalid public key");

    // Verify the signature
    let is_valid = sr25519::Pair::verify(&signature, message_signed, &public_key);

    if is_valid {
        Ok(())
    } else {
        Err("Invalid signature")
    }
}

fn get_rand_coordinates() -> (i32, i32) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(0..100), rng.gen_range(0..100))
}
