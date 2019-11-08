use crate::source_desc::SourceFileDesc;
use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    Syn(syn::Error),
    IO(std::io::Error),
    UnresolvedPathAttr(String),
    UnresolvedIncludeArg(String),
    MissingFile(SourceFileDesc),
}

#[derive(Debug)]
pub struct SourceError {
    pub file: PathBuf,
    pub error: Error,
}

impl SourceError {
    pub fn new(file: PathBuf, error: Error) -> Self {
        Self { file, error }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnresolvedPathAttr(ref path_attr) => {
                write!(f, "Unresolved path attribute in {}", path_attr)
            }
            Self::UnresolvedIncludeArg(ref include) => {
                write!(f, "Unresolved include argument in {}", include)
            }
            Self::MissingFile(ref path) => {
                write!(f, "File {:?} does not exist or could not be read", path)
            }
            Self::IO(ref cause) => write!(f, "I/O error: {}", cause),
            Self::Syn(ref cause) => write!(f, "Syn error: {}", cause),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IO(ref cause) => Some(cause),
            Self::Syn(ref cause) => Some(cause),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(cause: std::io::Error) -> Self {
        Self::IO(cause)
    }
}
impl From<syn::Error> for Error {
    fn from(cause: syn::Error) -> Self {
        Self::Syn(cause)
    }
}

impl From<Error> for Vec<Error> {
    fn from(cause: Error) -> Self {
        vec![cause]
    }
}
