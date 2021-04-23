use alpaca::{AlpacaMessage, Connection, WebSocket};
use anyhow::Result;
use futures::StreamExt;
use kafka_settings::producer;
use rdkafka::producer::FutureRecord;
use sentry_anyhow::capture_anyhow;
use std::time::Duration;
use tracing::{debug, error, info};

mod settings;
pub use settings::{AlpacaSettings, Settings};

async fn setup_websocket(settings: AlpacaSettings) -> Result<WebSocket> {
    let mut events: Vec<String> = vec![];
    if settings.trade_updates {
        events.push("trade_updates".into());
    }
    if settings.account_updates {
        events.push("account_updates".into());
    }

    Connection::new(
        settings.streaming_url,
        settings.key_id,
        settings.secret_key,
        events,
    )
    .connect()
    .await
    .map_err(|e| e.into())
}

pub async fn run(settings: Settings) -> Result<()> {
    let ws = setup_websocket(settings.alpaca).await?;
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
                            let e = e.into();
                            capture_anyhow(&e);
                            error!("Failed to send msg to Kafka: {:?}. Error: {}", msg, e)
                        }
                    }
                    Err(e) => {
                        let e = e.into();
                        capture_anyhow(&e);
                        error!("Failed to serialize payload: {:?}. Error: {}", &message, e)
                    }
                }
            }
            Err(e) => {
                let e = e.into();
                capture_anyhow(&e);
                error!("Failed to receive message from the WebSocket: {}", e)
            }
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
