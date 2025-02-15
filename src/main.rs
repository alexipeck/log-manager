use log_manager::{
    error::Error,
    logs::{Level, SimpleLog},
    manager::Pagination,
};
use serde::{Deserialize, Serialize};
use std::{
    io::stdout,
    time::{Duration, Instant},
};
use tracing::{debug, info};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer, Registry};
use uuid::{uuid, Uuid};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubSource {
    Toaster,
    Cat,
    Thermometer,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSource {
    Server,
    Agent(Uuid),
    SomeOtherSource(SubSource),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut prestart_logs: Vec<String> = Vec::new();
    prestart_logs.push("Starting Log Manager Example Server".to_string());
    let (stdout_writer, _guard) = tracing_appender::non_blocking(stdout());

    let level_filter = LevelFilter::from_level({
        let env_display_level = match std::env::var("LOG_MANAGER_DISPLAY_LEVEL") {
            Ok(level_str) => match level_str.to_lowercase().as_str() {
                "trace" => Some(tracing::Level::TRACE),
                "debug" => Some(tracing::Level::DEBUG),
                "info" => Some(tracing::Level::INFO),
                "warn" | "warning" => Some(tracing::Level::WARN),
                "error" | "err" => Some(tracing::Level::ERROR),
                _ => None,
            },
            Err(_) => None,
        };
        if env_display_level.is_none() {
            prestart_logs.push("ENV \"LOG_MANAGER_DISPLAY_LEVEL\" not set".to_string());
        }
        let display_level = env_display_level.unwrap_or(tracing::Level::INFO);
        prestart_logs.push(format!(
            "Running with display level: {}",
            display_level.to_string(),
        ));
        display_level
    });

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_writer(stdout_writer)
        .with_filter(level_filter);

    let subscriber = Registry::default().with(stdout_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    for log in prestart_logs {
        info!("{}", log);
    }

    info!("Running");
    let log_manager = log_manager::manager::Builder::default()
        .database_url("/data/indev_log_database.sql".into())
        .build::<LogSource>()
        .await?;
    /* {
        let results = log_manager.search(
            Some(LogSource::Agent(uuid!(
                "f068c603-b2d8-4aab-a06b-478dea93bcea"
            ))),
            None,
            "".into(),
        )?;
        debug!("Count: {}", results.len());
        for result in results {
            debug!("{:?}", result);
        }
    } */
    /* {
        let results = log_manager.search(
            Some(LogSource::Agent(uuid!(
                "f068c603-b2d8-4aab-a06b-478dea93bcea"
            ))),
            Some(Pagination::Page {
                page: 1,
                page_size: 2,
            }),
            "".into(),
        )?;
        //debug!("Count: {}", results.len());
        for result in results {
            debug!("{:?}", result);
        }
    } */
    for i in 1..50 {
        log_manager.save_log(
            SimpleLog::generate_log(Level::Info, "src/test".into(), i.to_string()),
            LogSource::Agent(uuid!("f068c603-b2d8-4aab-a06b-478dea93bcea")),
        )?;
        log_manager.save_log(
            SimpleLog::generate_log(Level::Debug, "src/test".into(), i.to_string()),
            LogSource::Agent(uuid!("f068c603-b2d8-4aab-a06b-478dea93bcea")),
        )?;
    }
    let (total_count, results) = log_manager.search(None, None, "".into(), &[Level::Debug])?;
    for i in 1..(total_count / 10) {
        let now = Instant::now();
        let (total_count, results) = log_manager.search(
            Some(LogSource::Agent(uuid!(
                "f068c603-b2d8-4aab-a06b-478dea93bcea"
            ))),
            Some(Pagination::Page {
                page: i as usize,
                page_size: 2,
            }),
            "".into(),
            &[Level::Debug],
        )?;
        debug!("Total before pagination: {total_count}");
        debug!("{}ns", now.elapsed().as_nanos());
        debug!("Page {i}");
        for result in results {
            debug!("{:?}", result);
        }
    }
    debug!("Search total (total before pagination: {total_count})");
    for result in results {
        debug!("{:?}", result);
    }
    log_manager.stop();
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}
