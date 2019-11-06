use crate::source_desc::SourceFileDesc;
use std::{fmt, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Unresolved {
    PathAttribute(String),
    IncludeArgument(String),
    MissingFile(SourceFileDesc),
}

#[derive(Debug)]
pub enum Error {
    Syn(syn::Error),
    IO(std::io::Error),
    Unresolved(Unresolved),
}

#[derive(Debug)]
pub struct SrcError {
    pub file: PathBuf,
    pub error: Error,
}

impl SrcError {
    pub fn new(file: PathBuf, error: Error) -> Self {
        Self { file, error }
    }
}

impl fmt::Display for Unresolved {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::PathAttribute(ref path_attr) => {
                write!(f, "Unresolved path attribute in {}", path_attr)
            }
            Self::IncludeArgument(ref include) => {
                write!(f, "Unresolved include argument in {}", include)
            }
            Self::MissingFile(ref path) => {
                write!(f, "File {:?} does not exist or could not be read", path)
            }
        }
    }
}

impl std::error::Error for Unresolved {}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::IO(ref cause) => write!(f, "I/O error: {}", cause),
            Self::Syn(ref cause) => write!(f, "Syn error: {}", cause),
            Self::Unresolved(ref cause) => write!(f, "Could not resolve item: {}", cause),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IO(ref cause) => Some(cause),
            Self::Syn(ref cause) => Some(cause),
            Self::Unresolved(ref cause) => Some(cause),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(cause: std::io::Error) -> Error {
        Self::IO(cause)
    }
}
impl From<syn::Error> for Error {
    fn from(cause: syn::Error) -> Error {
        Self::Syn(cause)
    }
}
impl From<Unresolved> for Error {
    fn from(cause: Unresolved) -> Error {
        Self::Unresolved(cause)
    }
}
