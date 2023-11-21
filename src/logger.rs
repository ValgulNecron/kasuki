use chrono::Utc;
use colored::Colorize;
use log::{Level, Log, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::path::Path;
use uuid::Uuid;

static LOGGER: SimpleLogger = SimpleLogger::new(Uuid::new_v4().to_string());

pub fn init_logger(log: &str) -> Result<(), SetLoggerError> {
    let level_filter = match log {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level_filter))
}

struct SimpleLogger {
    uuid: String,
}

impl SimpleLogger {
    pub fn new(uuid: String) -> Self {
        SimpleLogger { uuid }
    }
}
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn || metadata.target().starts_with("kasuki")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = match record.level() {
                Level::Error => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(230, 6, 6),
                Level::Warn => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(230, 84, 6),
                Level::Info => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(22, 255, 239),
                Level::Debug => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(106, 255, 0),
                Level::Trace => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(255, 0, 204),
            };

            match File::create(format!("log/{}", &self.uuid)) {
                Ok(_) => {}
                Err(_) => return,
            }

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(format!("log/log_{}.txt", &self.uuid))
                .unwrap();

            writeln!(file, "{}", text).unwrap();

            println!("{}", text);
        }
    }

    fn flush(&self) {}
}

pub fn remove_old_logs() -> Result<(), Error> {
    let path = Path::new("./");
    let mut entries: Vec<_> = fs::read_dir(&path)?.filter_map(Result::ok).collect();

    // Sort the entries by modification time
    entries.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());

    // Remove the oldest ones until there are only 5 left
    for entry in entries.iter().clone().into_iter().take(entries.len() - 5) {
        fs::remove_file(entry.path())?;
    }

    Ok(())
}

pub fn create_log_directory() -> std::io::Result<()> {
    fs::create_dir_all("log/")
}
