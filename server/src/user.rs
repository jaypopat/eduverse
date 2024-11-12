use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use mediasoup::consumer::Consumer;
use mediasoup::data_structures::{DtlsParameters, IceCandidate, IceParameters};
use mediasoup::prelude::{MediaKind, RtpParameters};
use mediasoup::producer::Producer;
use mediasoup::rtp_parameters::RtcpParameters;
use mediasoup::webrtc_transport::WebRtcTransport;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, ByteArray, Pair};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::room_manager::RoomManager;
use crate::ws_payload::{
    ConsumePayload, JoinPayload, MovementPayload, ProducePayload, ResumePayload, TransportOptions,
};
pub struct User {
    pub(crate) id: Option<String>,
    room_id: Option<u32>,
    coordinates: (i32, i32),
    pub websocket: WebSocketStream<TcpStream>,
    transport: Option<WebRtcTransport>,
    // Track what this user is broadcasting
    producers: HashMap<String, Producer>,
    // Track what this user is receiving
    consumers: HashMap<String, Consumer>,
    audio_range: f32,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
enum UserAction {
    // canvas movement/join/leave/send-message actions
    #[serde(rename = "join")]
    JoinRoom(JoinPayload),
    #[serde(rename = "leave")]
    LeaveRoom,
    #[serde(rename = "move")]
    MoveTo(MovementPayload),
    #[serde(rename = "send_message")]
    SendMessage(String),

    // webrtc actions
    #[serde(rename = "webrtc_init")]
    InitializeWebRTC, // client initializes a webrtc connection and the transporter
    #[serde(rename = "connect_transport")]
    // client connects to the sfu establishes a transport connection
    ConnectTransport(TransportOptions),
    #[serde(rename = "produce")] // when client wants to send video/audio to the sfu
    Produce(ProducePayload),
    #[serde(rename = "consume")] // when client wants to receive video/audio from the sfu
    Consume(ConsumePayload),
    #[serde(rename = "resume")]
    Resume(ResumePayload), // if the user paused a video to focus on audio-only, Resume would let them start receiving the video stream again.
}
impl User {
    pub fn new(mut websocket: WebSocketStream<TcpStream>) -> Self {
        User {
            id: None,
            room_id: None,
            coordinates: (0, 0),
            websocket,
            transport: None,
            producers: HashMap::new(),
            consumers: HashMap::new(),
            audio_range: 50.0,
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

                                UserAction::ConnectTransport(transport_options) => {
                                    Self::handle_connect_transport(
                                        user_arc.clone(),
                                        transport_options,
                                    )
                                    .await;
                                }
                                UserAction::Produce(produce_payload) => {
                                    Self::handle_produce(user_arc.clone(), produce_payload).await;
                                }
                                UserAction::Consume(consumePayload) => {
                                    Self::handle_consume(user_arc.clone(), consumePayload).await;
                                }
                                UserAction::Resume(resumePayload) => {
                                    Self::handle_resume(user_arc.clone(), resumePayload).await;
                                }
                                UserAction::InitializeWebRTC => {
                                    Self::handle_webrtc_init(user_arc.clone()).await;
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
        print!("Joining room");
        if !RoomManager::instance()
            .rooms
            .read()
            .await.contains_key(&payload.course_id)
        {
            return;
        }

        let (course_id, pub_address, signature, message_signed) = (
            payload.course_id,
            payload.pub_address,
            payload.signature,
            payload.message_signed,
        );
        //
        // // Verify the signature
        // if let Err(e) = verify_signature(pub_address.clone(), signature, message_signed) {
        //     eprintln!("Signature verification failed: {}", e);
        //     return;
        // }

        // TODO check if user is enrolled by calling is_enrolled view function in smart contract
        // const isEnrolled = contract.query.isEnrolled(course_id, pub_address);
        // if(!isEnrolled) eprintln!("You havent enrolled in: {}", e); return;

        let mut user = user_arc.lock().await;
        if user.id.is_none() {
            user.id = Some(pub_address.clone());
        }
        user.room_id = Some(course_id);
        user.coordinates = get_rand_coordinates();

        RoomManager::instance()
            .add_user_to_room(payload.course_id, user_arc.clone())
            .await;
        println!("User added to room");
        println!("{:?} {:?} {:?}", user.id, user.room_id, user.coordinates);

        let join_message = serde_json::json!({
            "type": "user_joined",
            "user_id": user.id,
            "coordinates": user.coordinates
        });

        RoomManager::instance()
            .broadcast_message(user.id.clone(), course_id, join_message.to_string())
            .await;
        print!("broadcasted to everyone");
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
    async fn handle_connect_transport(
        user_arc: Arc<Mutex<Self>>,
        transport_options: TransportOptions,
    ) {
        let mut user = user_arc.lock().await;

        unimplemented!("Connect transport not implemented yet");
    }
    async fn handle_webrtc_init(user_arc: Arc<Mutex<Self>>) {
        unimplemented!("WebRTC not implemented yet");
    }
    async fn handle_produce(user_arc: Arc<Mutex<Self>>, produce_payload: ProducePayload) {
        unimplemented!("Produce not implemented yet");
    }
    async fn handle_consume(user_arc: Arc<Mutex<Self>>, consume_payload: ConsumePayload) {
        unimplemented!("Consume not implemented yet");
    }
    async fn handle_resume(user_arc: Arc<Mutex<Self>>, consumer_id: ResumePayload) {
        unimplemented!("Resume not implemented yet");
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
// impl Drop for User {
//     fn drop(&mut self) {
//         self.producers.clear();
//         self.consumers.clear();
//
//         if let Some(mut transport) = self.transport.take() {
//             let _ = transport.close();
//         }
//     }
// }
