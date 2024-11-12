use sp_core::{Blake2Hasher, Hasher};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use subxt::ext::codec::{Decode, Encode};
use subxt::{OnlineClient, PolkadotConfig};
use tokio::sync::Mutex;

use crate::room_manager::RoomManager;
use sp_core::crypto::Ss58Codec;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

pub struct ContractClient {
    client: Arc<OnlineClient<PolkadotConfig>>,
    contract_address: subxt::config::polkadot::AccountId32,
}

impl ContractClient {
    pub async fn new(
        url: &str,
        contract_address: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let client = Arc::new(OnlineClient::<PolkadotConfig>::from_url(url).await?);
        let contract_address = subxt::config::polkadot::AccountId32::from_str(contract_address)?;

        Ok(Self {
            client,
            contract_address,
        })
    }

    // pub async fn is_enrolled(
    //     &self,
    //     course_id: u32,
    //     user_id: String,
    // ) -> Result<bool, Box<dyn Error + Send + Sync + 'static>> {
    //     let selector = get_selector("verifyEnrollment");
    //     let user_account_id = sp_core::crypto::AccountId32::from_ss58check(user_id.as_ref())
    //         .expect("Invalid address");
    //     let call_data = (user_account_id, course_id).encode();

    //     let call = subxt::tx::Payload::new(
    //         "Contracts",
    //         "call",
    //         (
    //             self.contract_address.clone(),
    //             0u128,       // value
    //             None::<u64>, // gas_limit (None means use the default)
    //             None,        // storage_deposit_limit
    //             selector.to_vec(),
    //             call_data,
    //         ),
    //     );

    //     let result = self.client.tx().call_dry_run(&call, None).await?;

    //     let is_enrolled: bool = result.decode()?;
    //     Ok(is_enrolled)
    // }
}

async fn create_room_websocket(teacher: String, course_id: u32, title: String) {
    println!("i was called as a result of smart contract call");
    let _ = RoomManager::instance()
        .add_room_from_contract(teacher, course_id, title.clone())
        .await;
    println!("Room created: {} - {}", course_id, title);
}

pub async fn listening_for_course_creations() -> Result<(), Box<dyn Error + Send + Sync + 'static>>
{
    println!("Listening for course creations...");

    let contract_client = Arc::new(Mutex::new(
        ContractClient::new(
            "wss://rpc2.paseo.popnetwork.xyz",
            "13CWQ2shoC3xjeEFUYsfbQT1gCUwpbJWtNJHhL4egjWheLAy",
        )
        .await?,
    ));

    let mut blocks_sub = contract_client
        .lock()
        .await
        .client
        .blocks()
        .subscribe_finalized()
        .await?;

    println!("Subscription to finalized blocks established.");

    while let Some(block_result) = blocks_sub.next().await {
        match block_result {
            Ok(new_block) => {
                let block_number = new_block.header().number;
                // println!("Processing block #{}", block_number);

                match new_block.events().await {
                    Ok(events) => {
                        for event in events.iter() {
                            if let Ok(event_details) = event {
                                if let Ok(Some(contract_event)) = event_details.as_event::<node_runtime::contracts::events::ContractEmitted>() {
                                    if contract_event.contract == contract_client.lock().await.contract_address {
                                        match decode_course_created_event(&contract_event.data) {
                                            Ok(course_created) => {
                                                println!("CourseCreated event: {:?}", course_created);
                                                let teacher_address = hex::encode(course_created.teacher);
                                                let title = String::from_utf8_lossy(&course_created.title).to_string();
                                                create_room_websocket(
                                                    teacher_address,
                                                    course_created.course_id,
                                                    title,
                                                ).await;
                                                println!("Room created for course: {}", course_created.course_id);
                                            }
                                            Err(e) => println!("Error decoding CourseCreated event: {:?}", e),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("Error fetching events for block {}: {:?}", block_number, e),
                }
            }
            Err(e) => println!("Error receiving block: {:?}", e),
        }
    }

    Ok(())
}

// pub async fn is_enrolled(
//     contract_client: &Arc<Mutex<ContractClient>>,
//     course_id: u32,
//     user_id: String,
// ) -> bool {
//     contract_client
//         .lock()
//         .await
//         .is_enrolled(course_id, user_id)
//         .await
//         .unwrap_or_else(|e| {
//             eprintln!("Error checking enrollment: {:?}", e);
//             false
//         })
// }

#[derive(Debug, Decode)]
struct CourseCreated {
    course_id: u32,
    teacher: [u8; 32],
    title: Vec<u8>,
}

fn decode_course_created_event(
    data: &[u8],
) -> Result<CourseCreated, Box<dyn Error + Send + Sync + 'static>> {
    let mut decoder = &data[..];
    CourseCreated::decode(&mut decoder).map_err(|e| e.into())
}

fn get_selector(name: &str) -> [u8; 4] {
    let mut result = Blake2Hasher::hash(name.as_bytes());

    let mut selector = [0u8; 4];
    selector.copy_from_slice(&result[0..4]);
    selector
}
