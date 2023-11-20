use chrono::Utc;
use colored::Colorize;
use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logger(log: &str) -> Result<(), SetLoggerError> {
    let level_filter = match log {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    log::set_logger(&LOGGER).map(|()| log::set_max_level(level_filter))
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn || metadata.target().starts_with("kasuki")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = match record.level() {
                Level::Error => {
                    format!("{} : {} - {}", Utc::now(), record.level(), record.args()).truecolor(230, 6, 6)
                }
                Level::Warn => {
                    format!("{} : {} - {}", Utc::now(), record.level(), record.args()).truecolor(230, 84, 6)
                }
                Level::Info => {
                    format!("{} : {} - {}", Utc::now(), record.level(), record.args()).truecolor(22, 255, 239)
                }
                Level::Debug => {
                    format!("{} : {} - {}", Utc::now(), record.level(), record.args()).truecolor(106, 255, 0)
                }
                Level::Trace => {
                    format!("{} : {} - {}", Utc::now(), record.level(), record.args()).truecolor(255, 0, 204)
                }
            };

            println!("{}", text);
        }
    }

    fn flush(&self) {}
}

