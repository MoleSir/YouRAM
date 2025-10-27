use crate::{charz::CharzError, circuit::CircuitError, pdk::PdkError, simulate::SimulateError};

#[derive(Debug, thiserror::Error)]
pub enum YouRAMError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    Circuit(#[from] CircuitError),

    #[error(transparent)]
    Simulate(#[from] SimulateError),

    #[error(transparent)]
    Charz(#[from] CharzError),

    #[error(transparent)]
    Pdk(#[from] PdkError),

    #[error("{0}")]
    Message(String),

    #[error("{msg} >> {err}")]
    Context { msg: String, err: Box<dyn std::error::Error> }
}

pub type YouRAMResult<T> = Result<T, YouRAMError>;

pub trait ErrorContext<T> {
    fn context<S: Into<String>>(self, msg: S) -> YouRAMResult<T>;
    fn with_context<S: Into<String>>(self, f: impl Fn() -> S) -> YouRAMResult<T>;
}

impl<T, E: std::error::Error + 'static> ErrorContext<T> for Result<T, E> {
    fn context<S: Into<String>>(self, msg: S) -> YouRAMResult<T> {
        self.map_err(|e| YouRAMError::Context { msg: msg.into(), err: Box::new(e) }) 
    }

    fn with_context<S: Into<String>>(self, f: impl Fn() -> S) -> YouRAMResult<T> {
        let msg = f();
        self.context(msg)
    }
}