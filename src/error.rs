use peck_lib::impl_error_wrapper;
use thiserror::Error;

impl_error_wrapper!(DieselConnectionError, diesel::result::ConnectionError);
impl_error_wrapper!(DieselResultError, diesel::result::Error);
impl_error_wrapper!(SerdeError, serde_json::error::Error);

#[derive(Error, Debug)]
pub enum Error {
    #[error("DieselConnection({0})")]
    DieselConnection(DieselConnectionError),
    #[error("DieselResult({0})")]
    DieselResult(DieselResultError),
    #[error("RunningMigrations({0})")]
    RunningMigrations(String),
    #[error("SerializingField({0}, {1})")]
    SerializingField(String, SerdeError),
    #[error("DeserializingField({0}, {1})")]
    DeserializingField(String, SerdeError),
    #[error("Builder({0})")]
    Builder(BuilderError),
    #[error("Errors({:?})", 0)]
    Errors(Vec<Self>),
}

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("MissingProperties({0})")]
    MissingProperties(String),
}
