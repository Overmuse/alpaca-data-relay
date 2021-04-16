use config::{Config, ConfigError, Environment};
use kafka_settings::KafkaSettings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AlpacaSettings {
    pub streaming_url: String,
    pub key_id: String,
    pub secret_key: String,
    pub account_updates: bool,
    pub trade_updates: bool,
}

#[derive(Debug, Deserialize)]
pub struct SentrySettings {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub alpaca: AlpacaSettings,
    pub kafka: KafkaSettings,
    pub sentry: SentrySettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(Environment::new().separator("__"))?;
        s.try_into()
    }
}
