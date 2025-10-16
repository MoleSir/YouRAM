use std::path::PathBuf;
use super::MeasError;

#[derive(Debug, thiserror::Error)]
pub enum SimulateError {
    #[error("times len '{0}' != Voltages len '{1}'")]
    TimesAndVoltageUnmatch(usize, usize),

    #[error("unsupport spice execute '{0}'")]
    UnsupportExecute(String),

    #[error("execute command '{0}' failed for '{1}'")]
    ExecuteError(String, String),

    #[error("invalid path '{0}'")]
    InvalidPath(PathBuf),

    #[error("meas error: '{0}'")]
    MeasError(#[from] MeasError),

    #[error("{msg} >> {err}")]
    Context { msg: String, err: Box<SimulateError> }
}   
