pub mod database;
pub mod error;
pub mod logs;
pub mod manager;
pub mod schema;

use std::sync::atomic::AtomicU32;

pub static NEXT_LOG_ID: AtomicU32 = AtomicU32::new(0);
