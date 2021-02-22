use alpaca::Connection;
use alpaca_data_relay::run;
use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = sentry::init(sentry::ClientOptions::new());
    dotenv()?;
    env_logger::builder().format_timestamp_micros().init();

    let mut events: Vec<String> = vec![];
    if env::var("TRADE_UPDATES").is_ok() {
        events.push("trade_updates".into());
    }
    if env::var("ACCOUNT_UPDATES").is_ok() {
        events.push("account_updates".into());
    }

    let ws = Connection::new(
        env::var("APCA_API_STREAMING_URL").context("Could not find APCA_API_STREAMING_URL")?,
        env::var("APCA_API_KEY_ID").context("Could not find APCA_API_KEY_ID")?,
        env::var("APCA_API_SECRET_KEY").context("Could not find APCA_API_SECRET_KEY")?,
        events,
    )
    .connect()
    .await?;

    run(ws).await;
    Ok(())
}
