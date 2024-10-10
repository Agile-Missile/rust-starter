use slog::o;
use slog::Drain;
#[cfg(all(target_os = "linux", feature = "journald"))]
use slog_journald::JournaldDrain;
#[cfg(feature = "syslog")]
use slog_syslog::Facility;

use super::error::Result;

pub fn setup_logging() -> Result<slog_scope::GlobalLoggerGuard> {
    // Setup Logging
    let guard = slog_scope::set_global_logger(default_root_logger()?);
    slog_stdlog::init()?;

    Ok(guard)
}

pub fn default_root_logger() -> Result<slog::Logger> {
    // Create drains
    let drain = slog::Duplicate(default_discard()?, default_discard()?).fuse();

    // Merge drains
    #[cfg(feature = "termlog")]
    let drain = slog::Duplicate(default_term_drain().unwrap_or(default_discard()?), drain).fuse();
    #[cfg(feature = "syslog")]
    let drain = slog::Duplicate(default_syslog_drain().unwrap_or(default_discard()?), drain).fuse();
    #[cfg(feature = "journald")]
    #[cfg(target_os = "linux")]
    let drain = slog::Duplicate(
        default_journald_drain().unwrap_or(default_discard()?),
        drain,
    )
    .fuse();

    // Create Logger
    let logger = slog::Logger::root(drain, o!("who" => "rust-starter"));

    // Return Logger
    Ok(logger)
}

fn default_discard() -> Result<slog_async::Async> {
    let drain = slog_async::Async::default(slog::Discard);

    Ok(drain)
}

// term drain: Log to Terminal
#[cfg(feature = "termlog")]
fn default_term_drain() -> Result<slog_async::Async> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let term = slog_term::FullFormat::new(plain);

    let drain = slog_async::Async::default(term.build().fuse());

    Ok(drain)
}

// syslog drain: Log to syslog
#[cfg(feature = "syslog")]
fn default_syslog_drain() -> Result<slog_async::Async> {
    let syslog = slog_syslog::unix_3164(Facility::LOG_USER)?;

    let drain = slog_async::Async::default(syslog.fuse());

    Ok(drain)
}

#[cfg(all(target_os = "linux", feature = "journald"))]
fn default_journald_drain() -> Result<slog_async::Async> {
    let journald = JournaldDrain.ignore_res();
    let drain = slog_async::Async::default(journald);

    Ok(drain)
}
