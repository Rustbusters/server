# Introduction
This repository provides the implementation for the chat server for the Advanced Programming Course 2024-2025 at the University of Trento.  

# Overview
The chat server implementation is organized in 2 main feature categories:

- **Standard** 🎃: basic features of `Network Discovery`, `Packet Handling`, and `Packet Source Routing`
- **Custom** 🤓: more advanced custom features that include:
    - **Persistency**: client messages are saved in a DB relative to each of the network servers.
    - **UI**: user interface for monitoring statistics and messages exchanged between clients.

## Persistency
Each server in the network includes a **DbManager**, responsible for handling message storage in a local **SQLite3** database.  
The **DbManager** performs the following operations:
- **Insertion**: Stores incoming messages from clients on the network persistently.
- **Retrieval**: Fetches messages when required for processing or visualization.
- **Deletion**: Removes old or processed messages when no longer needed.

## UI
The UI is an important component of this chat server implementation. It's able to display and monitor the server's statistics and clients' message exchanges.

The UI is implemented through the combination of a:

- **WebSocket Server**: Works as a bridge between the `Network Servers` and the `HTTP Server`.
- **HTTP Server**: Built using `tiny_http`, it serves a landing page that provides the UI for interacting with the servers.

This is a simple diagram explaining the overall architecture:
<img src="./assets/diagram.png" width="400px"/>
