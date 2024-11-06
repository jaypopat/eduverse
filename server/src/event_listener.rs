use std::error::Error;
use std::str::FromStr;
use subxt::{OnlineClient, PolkadotConfig};
use futures::StreamExt;
use sp_core::crypto::Ss58Codec;
use crate::room_manager::RoomManager;

async fn create_room_websocket(
    teacher: <PolkadotConfig as subxt::Config>::AccountId,
    course_id: u32,
    title: String,
) {
    RoomManager::instance()
        .add_room_from_contract(teacher, course_id, title.clone())
        .await;
    println!("Room created: {} - {}", course_id, title);
}

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

pub async fn listening_for_course_creations() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    println!("Listening for course creations...");

    // Connect to the Paseo Polkadot Testnet
    let client = OnlineClient::<PolkadotConfig>::from_url("wss://rpc2.paseo.popnetwork.xyz").await?;

    // Define your contract address
    let my_contract_address = "13CWQ2shoC3xjeEFUYsfbQT1gCUwpbJWtNJHhL4egjWheLAy";
    let contract_account_id = subxt::config::polkadot::AccountId32::from_str(my_contract_address).unwrap();

    // Subscribe to finalized blocks
    let mut blocks_sub = client.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let new_block = block?;
        let block_number = new_block.header().number;
        println!("New block #{block_number} created! ✨");

        // Get events for this block
        let events = new_block.events().await?;
        println!("Eventshiii: {:?}", events);

        // Filter for ContractEmitted events from your specific contract
        for event in events.find::<node_runtime::contracts::events::ContractEmitted>() {
            println!("Contract event1: {:?}", event);
            match event {
                Ok(contract_event) => {
                    println!("Contract event2: {:?}", contract_event);
                    if contract_event.contract == contract_account_id {
                        println!("Contract event received from our contract: {:?}", contract_event);
                        // Decode and process the event data
                        match decode_course_created_event(&contract_event.data) {
                            Ok(course_created) => {
                                println!("CourseCreated event: {:?}", course_created);
                                create_room_websocket(
                                    contract_account_id.clone(),
                                    course_created.course_id,
                                    course_created.title,
                                ).await;
                            },
                            Err(e) => println!("Error decoding CourseCreated event: {:?}", e),
                        }
                    }
                },
                Err(e) => println!("Error processing ContractEmitted event: {:?}", e),
            }
        }
    }

    Ok(())
}
use subxt::ext::codec::{Decode, Compact};
use subxt::utils::AccountId32;

#[derive(Decode,Debug)]
struct CourseCreated {
    course_id: u32,
    teacher: String,
    title: String,
}

fn decode_course_created_event(data: &[u8]) -> Result<CourseCreated, Box<dyn Error + Send + Sync + 'static>> {
    CourseCreated::decode(&mut &data[..]).map_err(|e| e.into())
}
