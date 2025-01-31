# Introduction
This is the chat server implementation for the Advanced Programming Course 2024-2025 at the University of Trento.  
This repository provides the implementation for the Communication server.

# Overview
The chat server implementation is organized in 2 main feature categories:

- **Standard** 🎃: basic features of Network Discovery, Packet Handling, and Packet Source Routing
- **Custom** 🤓: more advanced custom features that include:
    - Persistency: client messages are saved in a DB relative to each of the servers on the network
    - UI: user interface for interacting with the server and seeing statistics

## Persistency
Each server in the network includes a **DbManager**, responsible for handling message storage in a local **SQLite3** database.  
The **DbManager** performs the following operations:
- **Insertion**: Stores incoming and outgoing messages persistently.
- **Retrieval**: Fetches messages when required for processing or visualization.
- **Deletion**: Removes old or processed messages when no longer needed.

This ensures that message history is preserved, even if the server restarts.

## UI
The UI is implemented through the combination of:

- **WebSocket Server**: Enables full-duplex communication between the `n` Network Servers and the WebSocket Server. The WebSocket Server receives incoming messages via **crossbeam channels**, ensuring efficient message handling.
- **HTTP Server**: Built using `tiny_http`, it serves a landing page that provides the UI for interacting with the server and viewing statistics.

This is a simple diagram explaining the overall architecture:
<img src="./assets/diagram.png" width="400px"/>
