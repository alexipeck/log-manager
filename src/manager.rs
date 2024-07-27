use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use diesel::{
    dsl::max, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection, TextExpressionMethods,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};

use crate::{
    database::{establish_connection, model::LogModel, run_migrations, MIGRATIONS},
    error::{BuilderError, DieselResultError, Error},
    logs::{Log, SimpleLog},
    schema::log::{
        self as log_table, dsl::content as content_db, dsl::log as log_data,
        dsl::source as source_db,
    },
    serialize_or_return_err, NEXT_LOG_ID,
};

#[derive(Debug)]
pub enum RequiredProperties {
    DatabaseUrl,
}

pub struct Builder {
    //required
    database_url: Option<String>,

    //optional
    stop: Option<Arc<AtomicBool>>,
    stop_notify: Option<Arc<Notify>>,
    //defaulted
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            database_url: None,
            stop: None,
            stop_notify: None,
        }
    }
}

impl Builder {
    pub fn stop(mut self, stop: Arc<AtomicBool>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn stop_notify(mut self, stop_notify: Arc<Notify>) -> Self {
        self.stop_notify = Some(stop_notify);
        self
    }

    pub fn database_url(mut self, database_url: String) -> Self {
        self.database_url = Some(database_url);
        self
    }

    pub async fn build<S: Serialize + DeserializeOwned>(self) -> Result<Arc<LogManager<S>>, Error> {
        let mut missing_properties: Vec<RequiredProperties> = Vec::new();
        if self.database_url.is_none() {
            missing_properties.push(RequiredProperties::DatabaseUrl);
        }
        if !missing_properties.is_empty() {
            return Err(Error::Builder(BuilderError::MissingProperties(format!(
                "{:?}",
                missing_properties
            ))));
        }

        let stop: Arc<AtomicBool> = self.stop.unwrap_or(Arc::new(AtomicBool::new(false)));
        let stop_notify: Arc<Notify> = self.stop_notify.unwrap_or(Arc::new(Notify::new()));

        let log_manager: Arc<LogManager<S>> =
            LogManager::<S>::new(stop, stop_notify, self.database_url.unwrap()).await?;

        Ok(log_manager)
    }
}

fn get_next_log_id(database_url: &str) -> Result<u32, Error> {
    let mut connection = establish_connection(database_url)?;
    let max_id: i32 = match log_table::table
        .select(max(log_table::id))
        .first::<Option<i32>>(&mut connection)
    {
        Ok(max_id) => max_id.unwrap_or(0),
        Err(err) => panic!("{}", err),
    };

    if max_id.is_negative() {
        panic!("Log ID cannot be negative: {}", max_id);
    }

    Ok(max_id as u32)
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Pagination {
    Page { page: usize, page_size: usize },
}

pub struct LogManager<S: Serialize + DeserializeOwned> {
    stop: Arc<AtomicBool>,
    stop_notify: Arc<Notify>,
    database_url: String,
    _phantom: PhantomData<S>,
}
impl<S: Serialize + DeserializeOwned> LogManager<S> {
    //TODO: add an option on the builder which configures whether this server should stop with ctrl+c or wait for the stop signal
    async fn new(
        stop: Arc<AtomicBool>,
        stop_notify: Arc<Notify>,
        database_url: String,
    ) -> Result<Arc<Self>, Error> {
        info!("Running log manager database migrations");
        {
            let mut connection: SqliteConnection = establish_connection(&database_url)?;
            match run_migrations(&mut connection, MIGRATIONS) {
                Ok(_) => info!("Log manager database migrations ran succesfully"),
                Err(err) => return Err(Error::RunningMigrations(err.to_string())),
            }
        }
        NEXT_LOG_ID.store(get_next_log_id(&database_url)? + 1, Ordering::SeqCst);
        let manager = Arc::new(Self {
            stop,
            stop_notify,
            database_url,
            _phantom: PhantomData,
        });
        Self::start_server(manager.to_owned()).await;
        Ok(manager)
    }
    async fn start_server(manager: Arc<Self>) {
        tokio::task::spawn(async move {
            //
        });
    }

    pub fn save_log(&self, log: SimpleLog, source: S) -> Result<usize, Error> {
        let sqlite_connection = &mut establish_connection(&self.database_url)?;
        let log = LogModel::from(log, source)?;
        let insert_into = diesel::insert_into(log_table::table);
        match insert_into.values(log).execute(sqlite_connection) {
            Ok(rows_affected) => Ok(rows_affected),
            Err(err) => Err(Error::DieselResult(DieselResultError(err))),
        }
    }
    pub fn search(
        &self,
        source: Option<S>,
        pagination: Option<Pagination>,
        content_search: Option<&str>,
    ) -> Result<Vec<Log<S>>, Error> {
        let mut sqlite_connection = establish_connection(&self.database_url)?;
        let mut query = log_data.into_boxed();
        if let Some(source) = source {
            let source_serialized = serialize_or_return_err!(&source, "source");
            query = query.filter(source_db.eq(source_serialized))
        }
        if let Some(pagination) = pagination {
            match pagination {
                Pagination::Page { page, page_size } => {
                    query = query
                        .limit(page_size as i64)
                        .offset(((page - 1) * page_size) as i64)
                }
            }
        }
        if let Some(content_search) = content_search {
            query = query.filter(content_db.like(format!("%{content_search}%")));
        }
        match query.load::<LogModel>(&mut sqlite_connection) {
            Ok(log_models) => {
                //Not the most efficient way to do this
                let mut logs = Vec::new();
                let mut errors = Vec::new();
                log_models
                    .into_iter()
                    .for_each(|model| match Log::<S>::from(model) {
                        Ok(log_model) => logs.push(log_model),
                        Err(err) => errors.push(err),
                    });
                if !errors.is_empty() {
                    warn!("{}", Error::Errors(errors));
                }
                Ok(logs)
            }
            Err(err) => {
                let err = Error::DieselResult(DieselResultError(err));
                error!("{err}");
                return Err(err);
            }
        }
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.stop_notify.notify_waiters();
    }
}
