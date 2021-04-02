use alpaca_data_relay::{run, Settings};
use anyhow::Result;
use dotenv::dotenv;
use tracing::{info, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = sentry::init(sentry::ClientOptions::new());
    let _ = dotenv();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
    LogTracer::init().expect("Failed to set logger");
    let settings = Settings::new()?;
    info!("Starting alpaca-data-relay");

    run(settings).await
}
