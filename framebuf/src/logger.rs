use core::fmt::Write;
use log::{Level, Log};
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

use crate::{
    color::{self, Color},
    raw::write::RawWriter,
};

static LOGGER: Logger = Logger::new();

/// A thread-safe spin-based logger
#[derive(Default)]
pub struct Logger {
    writer: Mutex<Option<RawWriter>>,
}

// SAFETY: access to RawWriter is synchronized via Mutex
unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Logger {
    const fn new() -> Self {
        Self {
            writer: Mutex::new(None),
        }
    }
    /// Update the value inside the logger object.
    pub fn init(value: RawWriter) {
        let mut guard = LOGGER.writer.lock();
        _ = guard.replace(value);

        // Set the logger.
        log::set_logger(&LOGGER).unwrap(); // Can only fail if already initialized.

        // Set logger max level to level specified by log features
        log::set_max_level(log::STATIC_MAX_LEVEL);
    }
}

impl From<Level> for Color {
    fn from(value: Level) -> Self {
        match value {
            Level::Error => color::ERROR,
            Level::Warn => color::WARN,
            Level::Info => color::INFO,
            Level::Debug | Level::Trace => color::DEBUG,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        without_interrupts(|| {
            let mut guard = self.writer.lock();
            // logging is non-cirtical, we just return if the logger has not been initialized
            let Some(writer) = guard.as_mut() else {
                return;
            };
            let (old_fg, old_bg) = writer.colors();

            writer.set_colors(record.level().into(), old_bg);

            // first write log level
            writer
                .write_fmt(format_args!("[{}]: ", record.level()))
                .unwrap();

            // then write message
            writer.write_fmt(*record.args()).unwrap();

            writer.write_char('\n'); // new line

            writer.set_colors(old_fg, old_bg);
        });
    }

    fn flush(&self) {
        // TODO: add buffering
    }
}
