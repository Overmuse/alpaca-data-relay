use alpaca_data_relay::run;
use anyhow::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = sentry::init(sentry::ClientOptions::new());
    let _ = dotenv();
    env_logger::builder().format_timestamp_micros().init();

    run().await
}
