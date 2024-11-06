use mediasoup::rtp_parameters::MediaKind;

#[derive(Clone, Debug)]
pub enum StreamType {
    Camera,
    Audio,
}
#[derive(Clone, Debug)]
pub struct StreamInfo {
    // Who owns this stream (pubKey)
    user_id: String,
    stream_type: StreamType,  // e.g., Camera/Screen/Audio
    settings: StreamSettings, // Current stream settings
    position: (i32, i32),     // Position of the stream source (for spatial audio/video)
}
pub struct MediaState {
    stream_id: String,
    kind: MediaKind,
    quality: QualityLevel,
    active: bool,
}
#[derive(Clone, Debug)]
struct StreamSettings {
    quality: QualityLevel, // High/Medium/Low
    max_bitrate: u32,      // e.g., 1500000 for HD
    range: f32,            // e.g., 50.0 units - Range for spatial audio/video
}

#[derive(Clone, Debug)]
pub enum QualityLevel {
    High,   // 1080p
    Medium, // 720p
    Low,    // 480p
}
