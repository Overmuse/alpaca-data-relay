use alpaca::{AlpacaMessage, WebSocket};
use anyhow::{Context, Result};
use futures::StreamExt;
use log::{error, info};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::env;

pub async fn run(ws: WebSocket) {
    let producer = kafka_producer().unwrap();
    ws.for_each(|message| async {
        match message {
            Ok(message) => {
                info!("{:?}", &message);
                let topic = get_topic(&message);
                let key = get_key(&message);
                let payload = serde_json::to_string(&message);
                match payload {
                    Ok(payload) => {
                        producer.send(FutureRecord::to(topic).key(key).payload(&payload), 0);
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
    })
    .await;
}

pub fn kafka_producer() -> Result<FutureProducer> {
    ClientConfig::new()
        .set("bootstrap.servers", &env::var("BOOTSTRAP_SERVERS")?)
        .set("security.protocol", &env::var("SECURITY_PROTOCOL")?)
        .set("sasl.mechanisms", &env::var("SASL_MECHANISMS")?)
        .set("sasl.username", &env::var("SASL_USERNAME")?)
        .set("sasl.password", &env::var("SASL_PASSWORD")?)
        .set("enable.ssl.certificate.verification", "false")
        .set("message.timeout.ms", "5000")
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
