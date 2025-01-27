use common_utils::Stats;
use wg_2024::network::NodeId;

#[derive(Debug)]
pub enum InternalMessage {
    MessageSent(NodeId),
    FragmentSent(NodeId),
    MessageReceived(NodeId),
    FragmentReceived(NodeId),
    AckSent(NodeId),
    AckReceived(NodeId),
    NackReceived(NodeId),    
}

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