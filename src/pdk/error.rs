use reda_lib::error::LibError;

use super::Process;

#[derive(Debug, thiserror::Error)]
pub enum PdkError {
    #[error("un exit leaf cell '{0}'")]
    UnexitLeafCell(&'static str),

    #[error("expect {0} pins but got {1} in leaf cell '{2}'")]
    UnmatchLeafCellPinSize(usize, usize, &'static str),

    #[error("nmos model in process {0} not found")]
    NmosModelNotFound(Process),

    #[error("default operating conditions '{0}' not found")]
    DefaultOperatingConditionsNotFound(String),

    #[error("operating conditions not found in library")]
    OperatingConditionsNotFound,

    #[error("unkonw pg pin name {0}")]
    UnkownPgPinName(String),

    #[error("cell {0} not found in spice file not exit in lib file")]
    CellNotFoundInSpiceFile(String),

    #[error("can't get driver strenght in cell {0}")]
    CanNotGetDriverStrenghtInCell(String),

    #[error("lack port {0}")]
    LackPort(&'static str),

    #[error("expect attr {0} but no exit")]
    ExpectAttrButNotFound(&'static str),

    #[error(transparent)]
    Liberty(#[from] LibError),
}