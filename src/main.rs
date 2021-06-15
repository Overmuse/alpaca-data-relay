use alpaca_data_relay::{run, Settings};
use anyhow::Result;
use dotenv::dotenv;
use sentry_anyhow::capture_anyhow;
use tracing::{info, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let formatting_layer = BunyanFormattingLayer::new("alpaca-data-relay".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
    LogTracer::init().expect("Failed to set logger");
    let settings = Settings::new()?;
    let _guard = sentry::init((
        settings.sentry.address.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    info!("Starting alpaca-data-relay");

    let res = run(settings).await;
    if let Err(e) = res {
        capture_anyhow(&e);
    }
    Ok(())
}
