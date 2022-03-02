
///Result type encapsulating the [`error::Error type`]
pub type Result<T> = std::result::Result<T, Error>;

///Error type used to encapsulate all errors
#[derive(Debug)]
pub enum Error {
    Custom,
    AlreadyExists,
    IO(std::io::Error),
    FSExtra(fs_extra::error::Error),
    Zip(zip::result::ZipError),
    Serde(serde_json::Error)
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}
