mod service_unit;
mod socket_unit;
mod target_unit;
mod unit_parser;

pub use service_unit::*;
pub use socket_unit::*;
pub use target_unit::*;
pub use unit_parser::*;

use std::path::PathBuf;

pub struct ParsedCommonConfig {
    pub unit: ParsedUnitSection,
    pub install: ParsedInstallSection,
    pub name: String,
}
pub struct ParsedServiceConfig {
    pub common: ParsedCommonConfig,
    pub srvc: ParsedServiceSection,
}
pub struct ParsedSocketConfig {
    pub common: ParsedCommonConfig,
    pub sock: ParsedSocketSection,
}
pub struct ParsedTargetConfig {
    pub common: ParsedCommonConfig,
}

#[derive(Default)]
pub struct ParsedUnitSection {
    pub description: String,

    pub wants: Vec<String>,
    pub requires: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
#[derive(Clone)]
pub struct ParsedSingleSocketConfig {
    pub kind: crate::sockets::SocketKind,
    pub specialized: crate::sockets::SpecializedSocketConfig,
}

impl std::fmt::Debug for ParsedSingleSocketConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "SocketConfig {{ kind: {:?}, specialized: {:?} }}",
            self.kind, self.specialized
        )?;
        Ok(())
    }
}

pub struct ParsedSocketSection {
    pub sockets: Vec<ParsedSingleSocketConfig>,
    pub filedesc_name: Option<String>,
    pub services: Vec<String>,

    pub exec_section: ParsedExecSection,
}
pub struct ParsedServiceSection {
    pub restart: ServiceRestart,
    pub accept: bool,
    pub notifyaccess: NotifyKind,
    pub exec: Commandline,
    pub stop: Vec<Commandline>,
    pub stoppost: Vec<Commandline>,
    pub startpre: Vec<Commandline>,
    pub startpost: Vec<Commandline>,
    pub srcv_type: ServiceType,
    pub starttimeout: Option<Timeout>,
    pub stoptimeout: Option<Timeout>,
    pub generaltimeout: Option<Timeout>,

    pub dbus_name: Option<String>,

    pub sockets: Vec<String>,

    pub exec_section: ParsedExecSection,
}

#[derive(Default)]
pub struct ParsedInstallSection {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}
pub struct ParsedExecSection {
    pub user: Option<String>,
    pub group: Option<String>,
    pub stdout_path: Option<StdIoOption>,
    pub stderr_path: Option<StdIoOption>,
    pub supplementary_groups: Vec<String>,
    pub environment: Option<EnvVars>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ServiceType {
    Simple,
    Notify,
    Dbus,
    OneShot,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotifyKind {
    Main,
    Exec,
    All,
    None,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ServiceRestart {
    Always,
    No,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Timeout {
    Duration(std::time::Duration),
    Infinity,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StdIoOption {
    File(PathBuf),
    AppendFile(PathBuf),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum CommandlinePrefix {
    AtSign,
    Minus,
    Colon,
    Plus,
    Exclamation,
    DoubleExclamation,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Commandline {
    pub cmd: String,
    pub args: Vec<String>,
    pub prefixes: Vec<CommandlinePrefix>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EnvVars {
    pub vars: Vec<(String, String)>,
}

impl ToString for Commandline {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

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
                write!(f, "In file {:?}: Section {} is unknown", self.path, name)?;
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
