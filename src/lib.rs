pub const APP_NAME: &str = "whisp";
pub const APP_NAME_PRETTY: &str = "Whisp";
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;
pub mod event;
pub mod icon;
pub mod models;
pub mod notify;
pub mod process;
pub mod record;
