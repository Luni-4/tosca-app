use std::fs::OpenOptions;

use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, prelude::__tracing_subscriber_SubscriberExt,
};

const LOG_ERROR_FILENAME: &str = "log-error.log";
const LOG_DEBUG_FILENAME: &str = "log-debug.log";

pub(crate) fn create_subscriber() {
    let debug_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_DEBUG_FILENAME)
        .expect(&t!("logging.debug_file_error"));

    let error_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_ERROR_FILENAME)
        .expect(&t!("logging.file_error"));

    let console_subscriber = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(EnvFilter::new("debug"));

    let debug_subscriber = fmt::layer()
        .json()
        .with_writer(debug_file)
        .with_filter(EnvFilter::new("debug"));

    let error_subscriber = fmt::layer()
        .json()
        .with_writer(error_file)
        .with_filter(EnvFilter::new("error"));

    let subscriber = Registry::default()
        .with(console_subscriber)
        .with(debug_subscriber)
        .with(error_subscriber);

    tracing::subscriber::set_global_default(subscriber).expect(&t!("logging.subscriber_error"));
}
