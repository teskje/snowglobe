use std::any::Any;
use std::fmt;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for StringError {}

pub(crate) fn downcast(error: Box<dyn Any + Send>) -> Error {
    let error = match error.downcast::<String>() {
        Ok(s) => return Box::new(StringError(*s)),
        Err(e) => e,
    };
    let error = match error.downcast::<&'static str>() {
        Ok(s) => return Box::new(StringError(s.to_string())),
        Err(e) => e,
    };
    Box::new(StringError(format!("error: {error:?}")))
}
