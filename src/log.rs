use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn http_log_handler() {
    let date_format = "%Y-%m-%dT%H:%M:%S";
    let log = Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format(date_format),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    log
}
