use std::fmt;
use thiserror::Error;
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Unsupported primitive type `{type_str}`. Available types are defined by `json_trait_rs::PrimitiveType::VARIANTS`")]
    UnsupportedPrimitiveType { type_str: String },

#[derive(Debug, Error)]
pub struct ParseJsonError {
    pub kind: ParseJsonErrorKind,
    pub location: Option<(usize, usize)>,
}

impl fmt::Display for ParseJsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(location) = self.location {
			write!(f, "{} at ({}, {})", self.kind, location.0, location.1)
		} else {
			write!(f, "{}", self.kind)
		}
    }
}

#[derive(Debug, Error)]
pub enum ParseJsonErrorKind {
    #[error("Unsupported data type `{type_str}`")]
    UnsupportedDataType { type_str: String },
    #[error("RustType::from_json requires either json or serde_json")]
    ParserUnavailable,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Syntax error: `{description}`")]
    SyntaxError { description: String },
    #[error("`{description}`")]
    DataTypeMismatch { description: String },
    #[error("`{description}`")]
    UnexpectedEndOfJson { description: String },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error>),
}
