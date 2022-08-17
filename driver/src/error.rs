
#[derive(Debug, thiserror::Error)]
pub enum LibError {
    #[error("Other error")]
    Other
}