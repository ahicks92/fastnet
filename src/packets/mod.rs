pub use self::encoder::*;

mod encoder;

pub enum Packet {
    //Status request and response (channel -1)
    StatusRequest(StatusRequest),
    StatusResponse(StatusResponse),

    //Connection handshake (also channel -1).
    Connect,
    Aborted(String),
    Connected(u32),
    
    //Heartbeat (channel -2).
    Heartbeat(i16),
    
    //MTU estimation (-3 and -4)
    //These have to record the channel.
    ResetMTUCount {channel: i16},
    MTUCountWasReset {channel: i16},
    MTUEstimate {channel: i16, payload: Vec<u8>},
    MTUResponse {channel: i16, count: u32},
}

pub enum StatusRequest {
    FastnetQuery,
    VersionQuery,
    ExtensionQuery(String),
}

pub enum StatusResponse {
    FastnetResponse(u8),
    VersionResponse(String),
    ExtensionResponse {name: String, supported: u8},
}

//These are the channel constants.

//Status query and connection handshake channel.
pub const CONNECTION_CHANNEL: i16 = -1;

//Heartbeat channel.
pub const HEARTBEAT_CHANNEL: i16 = -2;

//Client and server MTU channels.
//These are named for the entity executing the MTU estimation algorithm, not for the entity receiving.
pub const MTU_SERVER_ESTIMATION_CHANNEL: i16 = -3;
pub const MTU_CLIENT_ESTIMATION_CHANNEL: i16 = -4;
