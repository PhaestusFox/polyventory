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

#[derive(Debug, thiserror::Error)]
pub enum DeError {
    #[error("The type {0} is not in the TypeRegistory")]
    TypeNotRegistered(String),
    #[error("{0} does not have ReflectSerialize data")]
    OpaqueNotSerde(&'static str),
    #[error("Path error {0}")]
    PathError(String),
    #[error("{0} can not be cast to full reflect")]
    NotFullReflect(&'static str),
    #[error("No Type/Data pair found at start of file: {start}..{end}")]
    NoTypeFound {
        start: usize,
        end: usize
    },
    #[error("Type {0} is ambiguous with multiple other types;\nFull path types are not currently supported")]
    AmbiguousType(String),
    #[error("Read Error: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("End of File")]
    EOF,
    #[error("Ron Failed to DeSerialize: {0}")]
    RonError(#[from] ron::de::Error),
    #[error("Found {0} items in seq expected {1}")]
    NotEnoughItems(usize, usize),
    #[error("Failed to convert Dynamic representation to Concrete Type {0}")]
    FromReflectFailed(&'static str),
    #[error("{0} Done not register FromReflect")]
    NoFromReflect(&'static str),

    #[error("UnknownVariant {0}::{1}")]
    UnknownVariant(&'static str, String),
}