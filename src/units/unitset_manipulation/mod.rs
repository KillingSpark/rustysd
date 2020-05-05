//! This module contains functions to modify the set of units, like starting a set of units, or removing units

mod activate;
mod deactivate;
mod insert_new;
mod locking;
mod remove;
mod sanity_check;

pub use activate::*;
pub use deactivate::*;
pub use insert_new::*;
pub use locking::*;
pub use remove::*;
pub use sanity_check::*;
