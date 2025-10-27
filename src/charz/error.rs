
#[derive(Debug, thiserror::Error)]
pub enum CharzError {
    #[error("lack function test config {0}")]
    LackFunctionTestConfigField(&'static str),
}