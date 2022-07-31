pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {}
    }
}
