use crate::{
    database::model::LogModel,
    error::{Error, SerdeError},
};
use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::{self, Debug};
use tracing::metadata::Level as TracingLevel;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl From<TracingLevel> for Level {
    fn from(value: TracingLevel) -> Self {
        match value {
            TracingLevel::DEBUG => Level::Debug,
            TracingLevel::ERROR => Level::Error,
            TracingLevel::INFO => Level::Info,
            TracingLevel::TRACE => Level::Trace,
            TracingLevel::WARN => Level::Warn,
        }
    }
}

impl From<&TracingLevel> for Level {
    fn from(value: &TracingLevel) -> Self {
        match *value {
            TracingLevel::DEBUG => Level::Debug,
            TracingLevel::ERROR => Level::Error,
            TracingLevel::INFO => Level::Info,
            TracingLevel::TRACE => Level::Trace,
            TracingLevel::WARN => Level::Warn,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Trace => "Trace",
                Self::Info => "Info",
                Self::Error => "Error",
                Self::Warn => "Warning",
                Self::Debug => "Debug",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Log<S> {
    source: S,
    timestamp: String,
    level: Level,
    location: String,
    content: String,
}

macro_rules! ok_or_return_err {
    ($t:expr, $field_name:expr) => {
        match $t {
            Ok(t) => t,
            Err(err) => {
                let err = Error::DeserializingField($field_name.to_string(), SerdeError(err));
                tracing::warn!("Error deserializing field {}: {err}", $field_name);
                return Err(err);
            }
        }
    };
}

impl<S: Serialize + DeserializeOwned> Log<S> {
    pub fn from(value: LogModel) -> Result<Log<S>, Error> {
        Ok(Self {
            source: ok_or_return_err!(serde_json::from_str(&value.source), "source"),
            timestamp: ok_or_return_err!(serde_json::from_str(&value.timestamp), "timestamp"),
            level: ok_or_return_err!(serde_json::from_str(&value.level), "level"),
            location: ok_or_return_err!(serde_json::from_str(&value.location), "location"),
            content: ok_or_return_err!(serde_json::from_str(&value.content), "content"),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleLog {
    pub timestamp: String,
    pub level: Level,
    pub location: String,
    pub content: String,
}

impl SimpleLog {
    fn new(timestamp: String, level: Level, location: String, content: String) -> Self {
        Self {
            timestamp,
            level,
            location,
            content,
        }
    }
    pub fn generate_log(level: Level, location: String, content: String) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            level,
            location,
            content,
        }
    }
}
