use std::fmt::{Display, Formatter};
use sedregex::ErrorKind;

///Result type encapsulating the [`error::Error type`]
pub type Result<T> = std::result::Result<T, Error>;

///Error type used to encapsulate all errors
#[derive(Debug)]
pub enum Error {
    AlreadyExists,
    DoesntExist,
    IO(std::io::Error),
    FSExtra(fs_extra::error::Error),
    Zip(zip::result::ZipError),
    Reqwest(reqwest::Error),
    Registry(registry::Error),
    SedRegex(sedregex::ErrorKind),
    SerdeJson(serde_json::Error),
    Win32API(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {

}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<fs_extra::error::Error> for Error {
    fn from(e: fs_extra::error::Error) -> Self {
        Error::FSExtra(e)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Error::Zip(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self { Error::Reqwest(e) }
}

impl From<registry::key::Error> for Error {
    fn from(e: registry::key::Error) -> Self { Error::Registry(registry::Error::from(e)) }
}

impl From<registry::value::Error> for Error {
    fn from(e: registry::value::Error) -> Self { Error::Registry(registry::Error::from(e)) }
}

impl From<sedregex::ErrorKind> for Error {
    fn from(e: ErrorKind) -> Self {
        Error::SedRegex(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}