use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let namespace = "<namespace>.servicebus.windows.net";
    let entity = "<hybrid-connection>";
    let sas_token = "<sas-token>";
    
    let args: Vec<String> = env::args().collect();
    let relay = &args[1];

    if relay.to_string() == "listener" {
        println!("Starting Listener");
        let _ = start_listener(namespace, entity, sas_token).await;
    }
    if relay.to_string() == "sender" {
        println!("Starting Sender");
        let _ = start_sender(namespace, entity, sas_token).await;
    }

    Ok(())
}

async fn start_listener(namespace: &str, entity: &str, sas_token: &str) -> Result<()>{
    let url_string = format!("wss://{}/$hc/{}?sb-hc-action=listen&sb-hc-token={}", namespace, entity, sas_token);
    let url = Url::parse(&url_string)?;
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Listener Connected");

    // Receiving messages from the server
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received message from server: {}", text);
                let test: serde_json::Value = serde_json::from_str(&text).unwrap();
                let request = &test["accept"];
                let target = &request["address"];
                let addr = target.to_string();
                let _ = rendezvous(&addr).await;
                println!("Client has disconnected");
            }
            _ => {}
        }
    }

    Ok(())
}

async fn start_sender(namespace: &str, entity: &str, sas_token: &str) -> Result<()>{
    let url_string = format!("wss://{}/$hc/{}?sb-hc-action=connect&sb-hc-token={}", namespace, entity, sas_token);
    let url = Url::parse(&url_string)?;
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Sender Connected");

    // Sending a message to the server
    let mut line = String::new();
    println!("Enter text: ");
    std::io::stdin().read_line(&mut line).unwrap();
    ws_stream.send(Message::Text(line.into())).await?;

    // Receiving messages from the server
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                println!("{}", text);
            }
            _ => {}
        }
        let mut line = String::new();
        println!("Enter text: ");
        std::io::stdin().read_line(&mut line).unwrap();
        ws_stream.send(Message::Text(line.into())).await?;
    }

    Ok(())
}

async fn rendezvous(target: &str) -> Result<()> {
    let target_rendezvous = target.replace("\"", "");
    println!("Client is connecting");
    let url_rendezvous: Url = Url::parse(&target_rendezvous)?;
    println!("URL: {}", url_rendezvous);
    let (mut ws_stream_rendezvous, _) = connect_async(url_rendezvous).await.expect("Failed to connect");
    println!("Client has connected");

    while let Some(msg) = ws_stream_rendezvous.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received message from server: {}", text);
                let response = ["Echo: ", &text].join("\n");
                ws_stream_rendezvous.send(Message::Text(response.into())).await?;
            }
            _ => {}    
        }
    }
    Ok(())
}