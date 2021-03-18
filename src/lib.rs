use alpaca::{AlpacaMessage, Connection, WebSocket};
use anyhow::{Context, Result};
use futures::StreamExt;
use log::{debug, error, info};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::env;
use std::time::Duration;

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

pub async fn run() -> Result<()> {
    let ws = setup_websocket().await?;
    let producer = kafka_producer()?;
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
                        producer
                            .send(
                                FutureRecord::to(topic).key(key).payload(&payload),
                                Duration::from_secs(0),
                            )
                            .await;
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

fn kafka_producer() -> Result<FutureProducer> {
    ClientConfig::new()
        .set("bootstrap.servers", &env::var("BOOTSTRAP_SERVERS")?)
        .set("security.protocol", &env::var("SECURITY_PROTOCOL")?)
        .set("sasl.mechanisms", &env::var("SASL_MECHANISMS")?)
        .set("sasl.username", &env::var("SASL_USERNAME")?)
        .set("sasl.password", &env::var("SASL_PASSWORD")?)
        .set("enable.ssl.certificate.verification", "false")
        .create()
        .context("Failed to create Kafka producer")
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
