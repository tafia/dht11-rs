#[derive(Debug)]
pub enum Error {
    Gpio(::cupi::Error),
    Time(::std::time::SystemTimeError),
    TimeOut
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::cupi::Error> for Error {
    fn from(error: ::cupi::Error) -> Error {
        Error::Gpio(error)
    }
}

impl From<::std::time::SystemTimeError> for Error {
    fn from(error: ::std::time::SystemTimeError) -> Error {
        Error::Time(error)
    }
}
