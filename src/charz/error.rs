
#[derive(Debug, thiserror::Error)]
pub enum CharzError {
    #[error("laack function test config {0}")]
    LackFunctionTestConfigFeild(&'static str),
}