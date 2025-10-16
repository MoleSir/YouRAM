#[derive(Debug, thiserror::Error)]
pub enum PdkError {
    #[error("un exit leaf cell '{0}'")]
    UnexitLeafCell(&'static str),

    #[error("expect {0} pins but got {1} in leaf cell '{2}'")]
    UnmatchLeafCellPinSize(usize, usize, &'static str),
}