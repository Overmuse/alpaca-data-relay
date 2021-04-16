use alpaca_data_relay::{run, Settings};
use anyhow::Result;
use dotenv::dotenv;
use sentry_anyhow::capture_anyhow;
use tracing::{info, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
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
