use crate::control;
use crate::control::CONTROL;
use crate::serial::SerialWriter;
use core::fmt;
use core::fmt::{Arguments, Write};
use log::{Level, LevelFilter, Log, Metadata, Record};
use ustyle::{Color, Style};

/// Initialize the logger with the given filter.
///
/// # Safety
/// The logger can only be initialized once.
pub unsafe fn init(max_level: LevelFilter) {
    unsafe {
        log::set_max_level_racy(max_level);
        log::set_logger_racy(&Logger).expect("failed to init logger");
    }
}

/// The global logger.
pub struct Logger;

impl Logger {
    /// Write the log to the given writer.
    pub fn write(
        &self,
        level: Level,
        args: &Arguments,
        writer: &mut dyn Write,
        error: Color,
        warn: Color,
        info: Color,
        debug: Color,
        trace: Color,
    ) -> fmt::Result {
        let style = Style::default().with_foreground(match level {
            Level::Error => error,
            Level::Warn => warn,
            Level::Info => info,
            Level::Debug => debug,
            Level::Trace => trace,
        });

        match level {
            Level::Error => {
                write!(writer, "[")?;
                style.style_to(writer, "error")?;
                write!(writer, "] ")?;
                style.style_fmt_to(writer, args)?;
            }

            Level::Warn => {
                write!(writer, "[")?;
                style.style_to(writer, "warn")?;
                write!(writer, "] ")?;
                style.style_fmt_to(writer, args)?;
            }

            Level::Info => {
                write!(writer, "[")?;
                style.style_to(writer, "info")?;
                write!(writer, "] ")?;
                write!(writer, "{args}")?;
            }

            Level::Debug => {
                write!(writer, "[")?;
                style.style_to(writer, "debug")?;
                write!(writer, "] ")?;
                write!(writer, "{args}")?;
            }

            Level::Trace => {
                write!(writer, "[")?;
                style.style_to(writer, "trace")?;
                write!(writer, "] ")?;
                style.style_fmt_to(writer, args)?;
            }
        }

        writeln!(writer)?;

        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = record.level();
        let args = record.args();

        writeln!(&mut SerialWriter, "[{}] {args}", level_to_str(level))
            .expect("Failed to write log to serial");

        if control::is_init() {
            CONTROL.get().run(|ctrl| {
                self.write(
                    level,
                    args,
                    ctrl,
                    Color::BrightRed,
                    Color::BrightYellow,
                    Color::BrighterBlue,
                    Color::BrightGreen,
                    Color::Purple,
                )
                .expect("failed to write log to control");
            });
        }
    }

    fn flush(&self) {}
}

/// Converts a [Level} to a static string.
///
/// This is used, because [Level::as_str] returns capitalized strings.
fn level_to_str(level: Level) -> &'static str {
    match level {
        Level::Error => "error",
        Level::Warn => "warn",
        Level::Info => "info",
        Level::Debug => "debug",
        Level::Trace => "trace",
    }
}
