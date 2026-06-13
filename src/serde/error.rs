#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[error("The type {0} is not in the TypeRegistory")]
    TypeNotRegistered(&'static str),
    #[error("Dynamic Types not supported")]
    IsDynamic,
    #[error("Wright Error: {0}")]
    WrightError(#[from] std::fmt::Error),
    #[error("{0} does not have ReflectSerialize data")]
    OpaqueNotSerde(&'static str),
    #[error("Path error {0}")]
    PathError(String),
    #[error("{0} can not be cast to full reflect")]
    NotFullReflect(&'static str),
}

impl<'a> From<bevy::reflect::ReflectPathError<'a>> for SerdeError {
    fn from(value: bevy::reflect::ReflectPathError) -> Self {
        Self::PathError(format!("{:?}", value))
    }
}