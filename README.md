# Introduction
This is the server implementation for the Advanced Programming Course 2024-2025 at the University of Trento.\
This repository provides the implementation for the Communication server.

# Overview
The server implementation is organized in 2 main feature blocks:

- Standard: basic features of Packet Handling and Network Discovery
- Custom: more advanced custom features that include:
    - Load balancer: server requests are split among multiple worker threads
    - Persistency: client messages are saved in a DB
    - UI: user interface for interacting with the server and seeing statistics

## Load balancer

## Persistency

## UI
The UI is implemented through the combination of:

- **WebSocket Server**: used for full-duplex communication between the servers + http client and the WebSocket Server.
- **HTTP Server**: allows the hosting of a landing page with the UI.

![[./assets/diagram.png]]
