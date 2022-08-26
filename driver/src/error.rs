
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Other error")]
    Other
}