use std::collections::HashMap;
use std::error::Error;
use std::num::{NonZeroU32, NonZeroU8};
use std::sync::Arc;
use std::time::Duration;
use futures_util::SinkExt;
use lazy_static::lazy_static;
use mediasoup::prelude::{RtpCodecParametersParameters, WorkerSettings};
use mediasoup::router::{Router, RouterOptions};
use mediasoup::rtp_parameters::{MimeTypeAudio, MimeTypeVideo, RtpCodecCapability};
use mediasoup::worker::{Worker, WorkerId};
use mediasoup::worker_manager::WorkerManager;
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use crate::stream_types::StreamInfo;
use crate::user::User;

lazy_static! {
    static ref ROOM_MANAGER: Arc<RoomManager> = {
        let manager = RoomManager::new();
        Arc::new(manager)
    };
}

pub struct Room {
    pub(crate) teacher: String,
    pub(crate) name: String,
    pub(crate) users: RwLock<Vec<Arc<Mutex<User>>>>,
    // each room has its own router
    router: Router,
    // Track all active streams in the room
    active_streams: HashMap<String, StreamInfo>,
    // Spatial grid for quick proximity checks
    spatial_grid: HashMap<(i32, i32), Vec<String>>,
    // Example: {
    //   (10,10): ["user1", "user2"],
    //   (11,10): ["user3"]
    // }
}

pub struct RoomManager {
    pub(crate) rooms: RwLock<HashMap<u32, Room>>,
    worker_manager: WorkerManager,
    worker_pool: Mutex<Vec<Worker>>,
    room_to_worker: Mutex<HashMap<u32, WorkerId>>,
}

impl RoomManager {
    pub async fn initialize_workers(&self) -> Result<(), Box<dyn Error>> {
        let mut pool = self.worker_pool.lock().await;
        for _ in 0..num_cpus::get() {
            let worker = self
                .worker_manager
                .create_worker(WorkerSettings::default())
                .await?;
            pool.push(worker);
        }
        Ok(())
    }

    fn new() -> Self {
        RoomManager {
            rooms: RwLock::new(HashMap::new()),
            worker_pool: Mutex::new(vec![]),
            worker_manager: WorkerManager::new(),
            room_to_worker: Mutex::new(HashMap::new()),
        }
    }
    pub async fn initialize(&self) -> Result<(), Box<dyn Error>> {
        self.initialize_workers().await?;
        Ok(())
    }
    pub fn instance() -> Arc<RoomManager> {
        ROOM_MANAGER.clone()
    }
    async fn get_least_loaded_worker(&self) -> Result<Worker, Box<dyn Error>> {
        let pool = self.worker_pool.lock().await;
        let worker_map = self.room_to_worker.lock().await;

        pool.iter()
            .min_by_key(|worker| {
                // Count rooms assigned to this worker
                worker_map
                    .values()
                    .filter(|&w_id| w_id == &worker.id())
                    .count()
            })
            .cloned()
            .ok_or_else(|| "No workers available".into())
    }

    pub async fn add_room_from_contract(
        &self,
        teacher: String,
        course_id: u32,
        course_name: String,
    ) -> Result<u32, Box<dyn Error>> {
        // Acquire a worker from the pool with the least load.
        let worker = self.get_least_loaded_worker().await?;
        let router = worker
            .create_router(RouterOptions::new(vec![
                RtpCodecCapability::Audio {
                    mime_type: MimeTypeAudio::Opus,
                    preferred_payload_type: None,
                    clock_rate: NonZeroU32::new(48000).unwrap(),
                    channels: NonZeroU8::new(2).unwrap(),
                    parameters: RtpCodecParametersParameters::default(),
                    rtcp_feedback: vec![],
                },
                RtpCodecCapability::Video {
                    mime_type: MimeTypeVideo::H264,
                    preferred_payload_type: None,
                    clock_rate: NonZeroU32::new(90000).unwrap(),
                    parameters: RtpCodecParametersParameters::default(),
                    rtcp_feedback: vec![],
                },
            ]))
            .await?;

        // Create a new Room instance.
        let room = Room {
            teacher: teacher.clone(),
            name: course_name.clone(),
            users: RwLock::new(vec![]),
            router,
            active_streams: Default::default(),
            spatial_grid: HashMap::new(),
        };

        // Lock and modify the rooms map.
        let mut rooms = self.rooms.write().await;
        rooms.insert(course_id, room);

        // Track room-worker association.
        let mut room_to_worker = self.room_to_worker.lock().await;
        room_to_worker.insert(course_id, worker.id());

        Ok(course_id)
    }

    pub(crate) async fn add_user_to_room(&self, room_id: u32, user: Arc<Mutex<User>>) {
        let mut rooms = self.rooms.write().await; // Use write lock for rooms
        if let Some(room) = rooms.get_mut(&room_id) {
            let mut users = room.users.write().await; // Use write lock for users
            users.push(user);
        }
    }

    pub(crate) async fn remove_user_from_room(&self, room_id: u32, user_id: String) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            let mut users = room.users.write().await;
            users.retain(|user| {
                let user_lock = futures::executor::block_on(user.lock());
                user_lock.id.as_ref() != Some(&user_id)
            });
        }
    }


}

impl RoomManager {
    pub(crate) async fn broadcast_message(
        &self,
        sender_id: Option<String>,
        room_id: u32,
        message: String,
    ) {
        let users = {
            let rooms = self.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                room.users.read().await.clone()
            } else {
                println!("Room {} not found", room_id);
                return;
            }
        };

        for user in users {
            match timeout(Duration::from_secs(5), user.lock()).await {
                Ok(mut user_lock) => {
                    let should_send = user_lock.id.as_ref()
                        .map(|user_id| sender_id.as_ref() != Some(user_id))
                        .unwrap_or(false);

                    if should_send {
                        if let Err(e) = user_lock.websocket.send(Message::Text(message.clone())).await {
                            if let Some(user_id) = &user_lock.id {
                                eprintln!("Failed to broadcast message to user {}: {}", user_id, e);
                            } else {
                                eprintln!("Failed to broadcast message to a user: {}", e);
                            }
                        }
                    }
                },
                Err(_) => {
                    eprintln!("Failed to acquire lock for a user in room {} within timeout", room_id);
                }
            }
        }
    }
}