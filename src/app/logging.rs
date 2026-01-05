//! logging stuff
use {
    crate::{config::options::LoggingFormat, getopt, utils::string_to_log_level},
    tracing_subscriber::FmtSubscriber,
};

/// setup logging
pub fn setup() -> color_eyre::Result<()> {
    let max_level = string_to_log_level(&getopt!(logging.level));
    let enabled = getopt!(logging.enable);

    if !enabled {
        return Ok(());
    }

    let subscriber = FmtSubscriber::builder()
        .with_max_level(max_level)
        .with_ansi(getopt!(logging.asni))
        .with_line_number(getopt!(logging.line_numbers))
        .with_target(getopt!(logging.event_targets));

    match getopt!(logging.format) {
        LoggingFormat::Pretty => {
            tracing::subscriber::set_global_default(subscriber.pretty().finish())?;
        }
        LoggingFormat::Compact => {
            tracing::subscriber::set_global_default(subscriber.compact().finish())?;
        }
    }

    tracing::info!("Logging setup successfully");
    Ok(())
}
