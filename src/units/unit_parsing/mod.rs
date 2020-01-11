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
    inner: ParsingErrorReason,
    path: std::path::PathBuf,
}

impl ParsingError {
    pub fn new(reason: ParsingErrorReason, path: std::path::PathBuf) -> ParsingError {
        ParsingError {
            inner: reason,
            path,
        }
    }
}

#[derive(Debug)]
pub enum ParsingErrorReason {
    UnknownSetting(String, String),
    UnusedSetting(String),
    UnsupportedSetting(String),
    MissingSetting(String),
    SettingTooManyValues(String, Vec<String>),
    SectionTooOften(String),
    SectionNotFound(String),
    UnknownSection(String),
    UnknownSocketAddr(String),
    FileError(Box<dyn std::error::Error>),
    Generic(String),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.inner {
            ParsingErrorReason::UnknownSetting(name, value) => {
                write!(
                    f,
                    "In file {:?}: setting {} was set to unrecognized value: {}",
                    self.path, name, value
                )?;
            }
            ParsingErrorReason::UnusedSetting(name) => {
                write!(
                    f,
                    "In file {:?}: unused setting {} occured",
                    self.path, name
                )?;
            }
            ParsingErrorReason::MissingSetting(name) => {
                write!(
                    f,
                    "In file {:?}: required setting {} missing",
                    self.path, name
                )?;
            }
            ParsingErrorReason::SectionNotFound(name) => {
                write!(
                    f,
                    "In file {:?}: Section {} wasn't found but is required",
                    self.path, name
                )?;
            }
            ParsingErrorReason::UnknownSection(name) => {
                write!(
                    f,
                    "In file {:?}: Section {} is unknown",
                    self.path, name
                )?;
            }
            ParsingErrorReason::SectionTooOften(name) => {
                write!(
                    f,
                    "In file {:?}: section {} occured multiple times",
                    self.path, name
                )?;
            }
            ParsingErrorReason::UnknownSocketAddr(addr) => {
                write!(
                    f,
                    "In file {:?}: Can not open sockets of addr: {}",
                    self.path, addr
                )?;
            }
            ParsingErrorReason::UnsupportedSetting(addr) => {
                write!(
                    f,
                    "In file {:?}: Setting not supported by this build (maybe need to enable feature flag?): {}",
                    self.path, addr
                )?;
            }
            ParsingErrorReason::SettingTooManyValues(name, values) => {
                write!(
                    f,
                    "In file {:?}: setting {} occured with too many values: {:?}",
                    self.path, name, values
                )?;
            }
            ParsingErrorReason::FileError(e) => {
                write!(f, "While parsing file {:?}: {}", self.path, e)?;
            }
            ParsingErrorReason::Generic(e) => {
                write!(f, "While parsing file {:?}: {}", self.path, e)?;
            }
        }

        Ok(())
    }
}

// This is important for other errors to wrap this one.
impl std::error::Error for ParsingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        if let ParsingErrorReason::FileError(err) = &self.inner {
            Some(err.as_ref())
        } else {
            None
        }
    }
}

impl std::convert::From<Box<std::io::Error>> for ParsingErrorReason {
    fn from(err: Box<std::io::Error>) -> Self {
        ParsingErrorReason::FileError(err)
    }
}
