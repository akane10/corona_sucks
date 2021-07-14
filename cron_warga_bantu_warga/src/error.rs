use std::fmt;

#[derive(Debug)]
pub enum Error {
    ReqError(reqwest::Error),
    IoError(std::io::Error),
    SerdejsonError(serde_json::Error),
    Others(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::ReqError(ref x) => write!(f, "{}", x),
            Error::IoError(ref x) => write!(f, "{}", x),
            Error::SerdejsonError(ref x) => write!(f, "{}", x),
            Error::Others(ref x) => write!(f, "{}", x),
        }
    }
}

impl std::error::Error for Error {}

macro_rules! error_wrap {
    ($f:ty, $e:expr) => {
        impl From<$f> for Error {
            fn from(f: $f) -> Error {
                $e(f)
            }
        }
    };
}

error_wrap!(reqwest::Error, Error::ReqError);
error_wrap!(std::io::Error, Error::IoError);
error_wrap!(serde_json::Error, Error::SerdejsonError);
