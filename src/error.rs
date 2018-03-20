use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    NotRunningWithSystemd,
    Io(io::Error),
}

const NOT_RUNNING_WITH_SYSTEMD: &str = "Not running with systemd";

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotRunningWithSystemd => write!(f, "{}", NOT_RUNNING_WITH_SYSTEMD),
            Error::Io(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        NOT_RUNNING_WITH_SYSTEMD
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::NotRunningWithSystemd => None,
            Error::Io(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
