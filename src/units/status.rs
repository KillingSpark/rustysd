use crate::units::*;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnitStatus {
    NeverStarted,
    Starting,
    Stopping,
    Restarting,
    Started(StatusStarted),
    Stopped(StatusStopped, Vec<UnitOperationErrorReason>),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum StatusStarted {
    Running,
    WaitingForSocket,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum StatusStopped {
    StoppedFinal,
    StoppedUnexpected,
}

impl UnitStatus {
    pub fn is_stopped(&self) -> bool {
        match self {
            UnitStatus::Stopped(_, _) => true,
            _ => false,
        }
    }
    pub fn is_started(&self) -> bool {
        match self {
            UnitStatus::Started(_) => true,
            _ => false,
        }
    }
}
