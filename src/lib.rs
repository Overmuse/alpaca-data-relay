use alpaca::{AlpacaMessage, Connection, WebSocket};
use anyhow::{Context, Result};
use futures::StreamExt;
use kafka_settings::producer;
use rdkafka::producer::FutureRecord;
use std::env;
use std::time::Duration;
use tracing::{debug, error, info};

mod settings;
pub use settings::Settings;

async fn setup_websocket() -> Result<WebSocket> {
    let mut events: Vec<String> = vec![];
    if env::var("TRADE_UPDATES").is_ok() {
        events.push("trade_updates".into());
    }
    if env::var("ACCOUNT_UPDATES").is_ok() {
        events.push("account_updates".into());
    }

    Connection::new(
        env::var("APCA_API_STREAMING_URL").context("Could not find APCA_API_STREAMING_URL")?,
        env::var("APCA_API_KEY_ID").context("Could not find APCA_API_KEY_ID")?,
        env::var("APCA_API_SECRET_KEY").context("Could not find APCA_API_SECRET_KEY")?,
        events,
    )
    .connect()
    .await
    .map_err(|e| e.into())
}

pub async fn run(settings: Settings) -> Result<()> {
    let ws = setup_websocket().await?;
    let producer = producer(&settings.kafka)?;
    ws.for_each_concurrent(None, |message| async {
        match message {
            Ok(message) => {
                let topic = get_topic(&message);
                let key = get_key(&message);
                let payload = serde_json::to_string(&message);
                match payload {
                    Ok(payload) => {
                        debug!(
                            "Message received: {}. Assigning key: {}, sending to topic: {}.",
                            &payload, &key, &topic
                        );
                        let send = producer
                            .send(
                                FutureRecord::to(topic).key(key).payload(&payload),
                                Duration::from_secs(0),
                            )
                            .await;
                        if let Err((e, msg)) = send {
                            error!("Failed to send msg to Kafka: {:?}. Error: {}", msg, e)
                        }
                    }
                    Err(e) => error!("Failed to serialize payload: {:?}. Error: {}", &message, e),
                }
            }
            Err(e) => error!("Failed to receive message from the WebSocket: {}", e),
        }
    })
    .await;
    info!("WebSocket closed, finalizing");
    Ok(())
}

fn get_topic(s: &AlpacaMessage) -> &str {
    match s {
        AlpacaMessage::TradeUpdates { .. } => "overmuse-trades",
        AlpacaMessage::AccountUpdates { .. } => "account",
        _ => "meta",
    }
}

fn get_key(s: &AlpacaMessage) -> &str {
    match s {
        AlpacaMessage::TradeUpdates(o) => &o.order.symbol,
        AlpacaMessage::AccountUpdates { .. } => "account",
        _ => "meta",
    }
}
