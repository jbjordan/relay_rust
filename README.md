# Relay Echo Listener and Sender in Rust
This repo contains an Echo websocket server using Azure Relay. With this example, you can start the relay listener on any machine, and start the relay sender on any machine. The relay sender will take command line input, send it to the listener over the websocket, and recieve an echo back.

Underlying websocket configuration is based on this repo: https://github.com/campbellgoe/rust_websocket_client

# Instructions
1. Update the relay namespace, entity, sas_key_name, and sas_key strings in main.rs with the values of your own Relay namespace information. Create a namespace: https://learn.microsoft.com/en-us/azure/azure-relay/relay-hybrid-connections-dotnet-get-started#create-a-namespace

2. Running the Relay Listener
First, ensure that you are in the directory of the project (rust_relay). 

Then, run the listener using Cargo:

```bash
cargo run -- listener
```
This will compile and start the relay listener. You should see output indicating that the listener has started:

```bash
Starting Listener
Listener Connected
```
3. Running the WebSocket Client
Open a new terminal window or tab. Navigate to the project directory (rust_relay).

Then, run the sender using Cargo:

```bash
cargo run -- sender
```
This will compile and start the relay sender. You should see output indicating that the client has connected to the server:

```bash
Starting Sender
Sender Connected
Enter Text:
```
4. If everything is set up correctly, you should be able to enter text and the listener will echo it back.

```bash
Starting Sender
Sender Connected
Enter text:
howdy
Echo:
howdy
```
### Additional Notes
Review Relay documentation to ensure machine can communicate with Relay server: https://learn.microsoft.com/en-us/azure/azure-relay/