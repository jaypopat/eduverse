use std::collections::HashMap;
use std::error::Error;
use std::num::{NonZeroU32, NonZeroU8};
use std::sync::Arc;

use futures_util::SinkExt;
use lazy_static::lazy_static;
use mediasoup::prelude::{RtpCodecParametersParameters, WorkerSettings};
use mediasoup::router::{Router, RouterOptions};
use mediasoup::rtp_parameters::{MimeTypeAudio, MimeTypeVideo, RtpCodecCapability};
use mediasoup::worker::{Worker, WorkerId};
use mediasoup::worker_manager::WorkerManager;
use subxt::{Config, SubstrateConfig};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::stream_types::StreamInfo;
use crate::user::User;

lazy_static! {
    static ref ROOM_MANAGER: Arc<RoomManager> = {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let manager = runtime.block_on(async {
            RoomManager::create()
                .await
                .expect("Failed to create RoomManager")
        });
        Arc::new(manager)
    };
}

pub struct Room {
    pub(crate) teacher: <SubstrateConfig as Config>::AccountId,
    pub(crate) name: String,
    pub(crate) users: Vec<Arc<Mutex<User>>>,
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
    pub(crate) rooms: Mutex<HashMap<u32, Room>>,
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
            rooms: Mutex::new(HashMap::new()),
            worker_pool: Mutex::new(vec![]),
            worker_manager: WorkerManager::new(),
            room_to_worker: Mutex::new(HashMap::new()),
        }
    }
    pub async fn create() -> Result<Self, Box<dyn Error>> {
        let manager = Self::new();
        manager.initialize_workers().await?;
        Ok(manager)
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
        teacher: <SubstrateConfig as Config>::AccountId,
        course_id: u32,
        name: String,
    ) -> Result<(), Box<dyn Error>> {
        let worker = self.get_least_loaded_worker().await?;

        let router_options = RouterOptions::new(vec![
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::Opus,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(48000).unwrap(),
                channels: NonZeroU8::new(2).unwrap(),
                parameters: RtpCodecParametersParameters::default(),
                rtcp_feedback: vec![],
            },
            RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::Vp8,
                preferred_payload_type: None,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: RtpCodecParametersParameters::default(),
                rtcp_feedback: vec![],
            },
        ]);

        let router = worker.create_router(router_options).await?;

        let mut rooms = self.rooms.lock().await;
        rooms.insert(
            course_id,
            Room {
                teacher,
                name,
                users: vec![],
                router,
                active_streams: Default::default(),
                spatial_grid: Default::default(),
            },
        );
        self.room_to_worker
            .lock()
            .await
            .insert(course_id, worker.id());

        Ok(())
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
                let mut user_lock = user.lock().await;
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
