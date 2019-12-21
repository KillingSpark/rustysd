mod service_unit;
mod socket_unit;
mod target_unit;
mod unit_parser;

pub use service_unit::*;
pub use socket_unit::*;
pub use target_unit::*;
pub use unit_parser::*;

#[derive(Debug)]
pub struct ParsingError {
    pub reason: Option<Box<dyn std::error::Error>>,
    pub msg: Option<String>,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "{}", msg)?;
        }
        if let Some(err) = &self.reason {
            write!(f, "source error:_{}", err)?;
        }

        Ok(())
    }
}

// This is important for other errors to wrap this one.
impl std::error::Error for ParsingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        if let Some(err) = &self.reason {
            Some(err.as_ref())
        } else {
            None
        }
    }
}

impl std::convert::From<String> for ParsingError {
    fn from(s: String) -> Self {
        ParsingError {
            reason: None,
            msg: Some(s),
        }
    }
}

impl std::convert::From<Box<dyn std::error::Error>> for ParsingError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ParsingError {
            reason: Some(err),
            msg: None,
        }
    }
}
