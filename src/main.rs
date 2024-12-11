extern crate base64;
extern crate hmac;
extern crate sha2;
extern crate urlencoding;

use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() -> Result<()> {
    let namespace = "<namespace>.servicebus.windows.net";
    let entity = "<entity>";
    let sas_key_name = "<sas-key-name>";
    let sas_key = "<sas-key>";

    let sas_token_temp = create_sas_token(namespace, entity, sas_key_name, sas_key);
    let sas_token = urlencoding::encode(&sas_token_temp);

    let args: Vec<String> = env::args().collect();
    let relay = &args[1];

    if relay.to_string() == "listener" {
        println!("Starting Listener");
        let _ = start_listener(namespace, entity, &sas_token).await;
    }
    if relay.to_string() == "sender" {
        println!("Starting Sender");
        let _ = start_sender(namespace, entity, &sas_token).await;
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
                println!("Received Sender Request:");
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

fn create_sas_token(service_namespace: &str, entity_path: &str, sas_key_name: &str, sas_key: &str) -> String {
    let uri = format!("http://{}/{}", service_namespace, entity_path);
    let encoded_resource_uri = urlencoding::encode(&uri);

    let token_valid_time_in_seconds = 60 * 60 * 48; // 48 hours
    let unix_seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let expiry_in_seconds = unix_seconds + token_valid_time_in_seconds;

    let plain_signature = format!("{}\n{}", encoded_resource_uri, expiry_in_seconds);
    let sas_key_bytes = sas_key.as_bytes();
    let plain_signature_bytes = plain_signature.as_bytes();
    let mut mac = HmacSha256::new_from_slice(sas_key_bytes).expect("HMAC can take key of any size");
    mac.update(plain_signature_bytes);
    let hash_bytes = mac.finalize().into_bytes().to_vec();
    let base64_hash_value = base64::encode(hash_bytes);

    let token = format!(
        "SharedAccessSignature sr={}&sig={}&se={}&skn={}",
        encoded_resource_uri,
        urlencoding::encode(&base64_hash_value),
        expiry_in_seconds,
        sas_key_name
    );

    token
}