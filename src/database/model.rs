use diesel::{Identifiable, Insertable, Queryable};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::atomic::Ordering;

use crate::error::Error;
use crate::schema::log;
use crate::{logs::SimpleLog, NEXT_LOG_ID};

#[macro_export]
macro_rules! serialize_or_return_err {
    ($t:expr, $field_name:expr) => {
        match serde_json::to_string(&$t) {
            Ok(t) => t,
            Err(err) => {
                let err = crate::error::Error::SerializingField(
                    $field_name.to_string(),
                    crate::error::SerdeError(err),
                );
                tracing::warn!("Error serializing field {}: {err}", $field_name);
                return Err(err);
            }
        }
    };
}

#[derive(Insertable, Queryable, Identifiable)]
#[diesel(primary_key(id))]
#[diesel(table_name = log)]
pub struct LogModel {
    pub id: i32,
    pub source: String,
    pub timestamp: String,
    pub level: String,
    pub location: String,
    pub content: String,
}

impl LogModel {
    pub fn from<S: Serialize + DeserializeOwned>(
        value: SimpleLog,
        source: S,
    ) -> Result<Self, Error> {
        Ok(Self {
            id: NEXT_LOG_ID.fetch_add(1, Ordering::SeqCst) as i32,
            source: serialize_or_return_err!(&source, "source"),
            timestamp: serialize_or_return_err!(&value.timestamp, "timestamp"),
            level: serialize_or_return_err!(&value.level, "level"),
            location: serialize_or_return_err!(&value.location, "location"),
            content: serialize_or_return_err!(&value.content, "content"),
        })
    }
}
