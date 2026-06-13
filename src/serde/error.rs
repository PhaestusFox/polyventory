#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[error("The type {0} is not in the TypeRegistory")]
    TypeNotRegistered(&'static str),
    #[error("Dynamic Types not supported")]
    IsDynamic,
    #[error("Wright Error: {0}")]
    WrightError(#[from] std::fmt::Error)
}