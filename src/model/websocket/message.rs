use common_utils::Stats;

pub enum WebSocketMessage {
    FromClient(ClientToServerMessage),
    FromServer(ServerToClientMessage),
}

// Client -> Server WebSocket messages
pub enum ClientToServerMessage {
    RegisterClient,
    UnregisterClient,
    PushStats(Stats),
    Text(String)
}

// Server -> Client WebSocket messages
pub enum ServerToClientMessage { 
    RegistrationSuccess,
    RegistrationFailure,
    ForwardStats(Stats),
    Text(String)
}