use mediasoup::data_structures::{DtlsParameters, IceCandidate, IceParameters};
use mediasoup::prelude::{MediaKind, RtcpParameters, RtpParameters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MovementPayload {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
#[derive(Deserialize)]
pub struct JoinPayload {
    pub(crate) course_id: u32,
    pub(crate) pub_address: String,
    pub(crate) signature: String,
    pub(crate) message_signed: String,
}
#[derive(Deserialize)]
pub struct WebRTCConnectPayload {
    dtls_parameters: RtcpParameters,
}

#[derive(Deserialize)]
pub struct ProducePayload {
    kind: MediaKind,
    rtp_parameters: RtpParameters,
}

#[derive(Deserialize)]
pub struct ConsumePayload {
    producer_id: String,
}
#[derive(Deserialize)]
pub struct ResumePayload {
    consumer_id: String,
}
#[derive(Deserialize)]
pub struct TransportOptions {
    id: String,
    ice_parameters: IceParameters,
    ice_candidates: Vec<IceCandidate>,
    dtls_parameters: DtlsParameters,
}
