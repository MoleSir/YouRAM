#[derive(Debug, thiserror::Error)]
pub enum CircuitError {
    #[error("port '{0}' already exit")]
    AddDuplicatePort(String),

    #[error("instance '{0}' already exit")]
    AddDuplicateInstance(String),

    #[error("unmatch pin size '{0}' and net size '{1}'")]
    PinSizeUnmatch(usize, usize),

    #[error("instance '{0}' have not been connected")]
    InstanceNotConnected(String),

    #[error("circuit arguments invalid: {0}")]
    InvalidArguments(String),

    #[error("no exit port {0} in circuit {1}")]
    PortNotFound(String, String),

    #[error("no exit pin {0} in instance {1}")]
    PinNotFound(String, String),

    #[error("no exit instance {0} in module {1}")]
    InstanceNotFound(String, String),

    #[error("{0}")]
    Messgae(String),
}

impl CircuitError {
    pub fn invalid_arg<S: Into<String>>(msg: S) -> Self {
        Self::InvalidArguments(msg.into())
    }

    pub fn msg<S: Into<String>>(msg: S) -> Self {
        Self::Messgae(msg.into())
    }
}

#[macro_export]
macro_rules! invalid_arg {
    ($msg:literal $(,)?) => {
        Err($crate::circuit::CircuitError::InvalidArguments(format!($msg).into()))?
    };
    ($err:expr $(,)?) => {
        Err($crate::circuit::CircuitError::InvalidArguments(format!($err).into()))?
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err($crate::circuit::CircuitError::InvalidArguments(format!($fmt, $($arg)*).into()))?
    };
}