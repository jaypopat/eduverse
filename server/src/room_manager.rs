use std::collections::HashMap;
use std::sync::Arc;

use futures_util::SinkExt;
use lazy_static::lazy_static;
use subxt::{Config, SubstrateConfig};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::user::User;

lazy_static! {
    static ref ROOM_MANAGER: Arc<RoomManager> = Arc::new(RoomManager::new());
}

pub struct Room {
    pub(crate) teacher: <SubstrateConfig as Config>::AccountId,
    pub(crate) name: String,
    pub(crate) users: Vec<Arc<Mutex<User>>>,
}

pub struct RoomManager {
    pub(crate) rooms: Mutex<HashMap<u32, Room>>,
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: Mutex::new(HashMap::new()),
        }
    }
    pub fn instance() -> Arc<RoomManager> {
        ROOM_MANAGER.clone()
    }
    pub async fn add_room_from_contract(
        &self,
        teacher: <SubstrateConfig as Config>::AccountId,
        course_id: u32,
        name: String,
    ) {
        let mut rooms = self.rooms.lock().await;
        rooms.insert(
            course_id,
            Room {
                teacher,
                name,
                users: vec![],
            },
        );
    }
    pub(crate) async fn add_user_to_room(&self, room_id: u32, user: Arc<Mutex<User>>) {
        if let Some(room) = self.rooms.lock().await.get_mut(&room_id) {
            room.users.push(user);
        }
    }
    pub(crate) async fn remove_user_from_room(&self, room_id: u32, user_id: String) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            let mut i = 0;
            while i < room.users.len() {
                let should_remove = {
                    let user_lock = room.users[i].lock().await;
                    user_lock.id == Some(user_id.clone())
                };
                if should_remove {
                    room.users.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    pub(crate) async fn broadcast_message(
        &self,
        sender_id: Option<String>,
        room_id: u32,
        message: String,
    ) {
        let rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(&room_id) {
            for user in &room.users {
                if let mut user_lock = user.lock().await {
                    if user_lock.id != sender_id {
                        user_lock
                            .websocket
                            .send(Message::Text(message.clone()))
                            .await
                            .expect("Error sending message");
                    }
                }
            }
        }
    }
}
