use crate::source_desc::SourceFileDesc;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Syn(syn::Error),
    IO(std::io::Error),
    UnresolvedPathAttr(String),
    UnresolvedIncludeArg(String),
    MissingFile(SourceFileDesc),
}

#[derive(Debug)]
pub struct SourcesAndErrors {
    pub sources: Vec<(SourceFileDesc, Vec<Error>)>,
}

impl SourcesAndErrors {
    pub fn new(sources: Vec<(SourceFileDesc, Vec<Error>)>) -> Self {
        Self { sources }
    }

    pub fn into_sources(self) -> Vec<SourceFileDesc> {
        self.sources.into_iter().map(|x| x.0).collect()
    }

    pub fn into_errors(self) -> Vec<(SourceFileDesc, Error)> {
        let mut result = vec![];

        for i in self.sources {
            let source_desc = i.0;
            result.extend(i.1.into_iter().map(|x| (source_desc.clone(), x)));
        }

        result
    }

    pub fn get_sources(&self) -> Vec<SourceFileDesc> {
        self.sources.iter().map(|x| x.0.clone()).collect()
    }
}

impl fmt::Display for SourcesAndErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.sources {
            if i.1.is_empty() {
                writeln!(f, "{}", i.0.path.display())?;
            } else {
                writeln!(f, "Errors in {}: ", i.0.path.display())?;

                for err in &i.1 {
                    writeln!(f, "{:2}", err)?;
                }
            }
        }

        Ok(())
    }
}

impl std::error::Error for SourcesAndErrors {}

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
