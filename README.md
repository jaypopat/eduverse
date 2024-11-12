# Eduverse: Metaverse Classroom: Architecture & Flow

## System Overview
Our Metaverse classroom platform integrates blockchain-based course management with real-time communication and spatial media streaming.

## Detailed Flow

### 1. Course Creation & Enrollment
- Teachers create courses through Ink! smart contracts, setting fees and parameters
- Event indexer monitors blockchain for course creation events and automatically provisions virtual classrooms
- Students can enroll by paying the specified fee through the smart contract
- Enrollment status is permanently recorded on-chain for access control

### 2. Secure Room Access
- Students connect to the virtual classroom via WebSocket
- Authentication uses blockchain wallet signatures:
    - Student signs a text with their private key
    - Backend verifies signature and checks enrollment status
    - Access granted only to verified, enrolled students

### 3. Virtual Classroom Interaction
- Students join an interactive canvas representing the virtual classroom
- Real-time interactions via WebSocket:
    - Movement using WASD/arrow keys
    - Text messaging
    - Position synchronization across all participants
- All actions are broadcast to relevant participants, creating a shared space

### 4. Audio/Video Communication (SFU Architecture)
Our Selective Forwarding Unit (SFU) enables efficient, spatial audio/video:

1. **Connection Setup**
    - Student initiates WebRTC connection
    - Transport established with ICE/DTLS parameters
    - Secure media channel created

2. **Media Streaming**
    - **Production**: Students sharing audio/video create producer transports
    - **Consumption**: Students receiving create consumer transports
    - **Routing**: SFU intelligently forwards streams based on:
        - Virtual proximity
        - Classroom layout
        - Network conditions

3. **Spatial Features**
    - Audio volume adjusts based on virtual distance
    - Video quality scales with proximity
    - Stream forwarding limited to relevant participants
    - Resource optimization through selective streaming

## Technical Implementation
- One worker per CPU core for efficient processing
- One router per virtual classroom
- Spatial grid system for proximity calculations
- Automatic quality and bandwidth management

![image](https://github.com/user-attachments/assets/87b70990-a3fe-4a5b-82cf-9b92ea788839)
