# Introduction
This repository provides the RustBusters implementation for the chat server for the Advanced Programming Course 2024-2025 at the University of Trento.

# Overview
The chat server implementation is organized in 2 main sets of features:

- **Standard** 🎃: basic features of `Network Discovery`, `Packet Handling`, and `Packet Source Routing`
- **Custom** 🤓: more advanced custom features that include:
    - **Persistency**: client messages are saved in a local database on the network servers.
    - **UI**: user interface for monitoring statistics, messages exchanged between clients and active users.

## Infrastructure
The Server provides 2 main components:
- **RustBustersServerController**: this one handles 2 important `Service`s running on separate threads: 
    - The `http` server for handling requests from the server's frontend and serving static content (e.g. images,*.css, *.js).
    - The `websocket` server for handling message forwarding from the servers on the network, or *network servers*, to the websocket server.
- **Network Server**: this element represents the server on the network, it provides basic functionality `Network Discovery`, `Packet Handling`, and `Packet Source Routing` and a persistency mechanism.

## Persistency
Each server in the network includes an attribute called `db_manager` which is an instance of the **DbManager** struct. This component is responsible for handling message storage in a local **SQLite3** relational database.  
The **DbManager** performs the following operations:
- **Insertion**: Stores incoming messages from clients on the network persistently.
- **Retrieval**: Fetches messages when required for processing or visualization.
- **Deletion**: Removes old or processed messages when no longer needed.

The structure of the messages is the following:
```rust
struct DbMessage {
    id: String, // UUID as the primary key
    src_id: NodeId, // sender id
    dest_id: NodeId, // recepient id
    message: String, // content, can be text/image
    timestamp: i64, // Unix timestamp
}
```

## UI
The UI is an important component of this chat server implementation. It's able to display the server's statistics, clients' message exchanges and active users. All of this in in real time!

The UI is implemented through the combination of a:

- **WebSocket Server**: Works as a bridge between the `Network Server`s and the `HTTP Server`.
- **HTTP Server**: Built using `tiny_http`, it serves a landing page that provides the UI for interacting with the servers and handles UI's requests for statistics, messages and active users.

Example:
1. Let's say that a user wants to retrieve specific server information, like the server's statistics for example.
2. The UI client makes a request to the `HTTP Server` on the endpoint `/api/servers/stats/:serverId`.
3. The `HTTP Server` handles the request, then the `WSChannelsManager` selects the crossbeam channel for the specified `Network Server` and forwards the request to the server by sending a `WebSocketRequest` through the channel.
4. The `Network Server` receives the `WebSocketRequest`, handles it and forwards the response to the `WebSocket Server`.
5. The `WebSocket Server` listens for incoming `InternalMessage`s, receives them and constructs a response to forward to the UI.
6. The websocket client on the UI listens for incoming messages and update the UI accordingly.


This is a simple diagram explaining the overall architecture:
<img src="./assets/diagram.png" />
