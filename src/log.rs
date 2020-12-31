use anyhow::{Context, Result};
use chrono::offset::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} {:<5} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) -> Result<()> {
    log::set_logger(&Logger).context("Failed to set logger")?;
    log::set_max_level(level);

    Ok(())
}
