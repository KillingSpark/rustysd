//! This module should provide all platfrom specific code.
//! All calls to the libc:: crate should be encapsulated here, so they can be replaced with calls to syscall:: for redoxos for example
//! Right now calls to the nix:: crate are not yet in here but they should be.
//!
//! unix_common should contain all code that deals with the functionality provided by all unixes
//!
//! subreaper should contain an implementation that sets a process as the subreaper for the current process tree
//! (not sure what should happen if the platform doesnt provide this feature)
//!
//! eventfd should contain an implementation that creates an eventfd (or a similarly working) tuple of filedescriptors
//! The pipe() implementation should work (in some variation) on many platforms
//!
//! ## Redox support
//! To implement all this stuff in redox we probably need these crates:
//! 1. relibc (for the select, which is not yet in the syscalls crate?)
//! 2. syscall (for most of all other nix:: functions)
//!
//! We could also wait for [this pull request](https://github.com/nix-rust/nix/pull/1098) to the nix crate to get redox support in there which would
//! make our life very much easier
//!
//! We'd also need to make some more functionality optional like subprocess reaping (which only matters if we are not PID1)
//!

mod drop_privileges;
mod eventfd;
mod subreaper;
mod unix_common;

pub use drop_privileges::*;
pub use eventfd::*;
pub use subreaper::*;
pub mod grnam;
pub mod pwnam;

//#[cfg(feature = "cgroups")]
pub mod cgroups;

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "linux"
))]
pub use unix_common::*;
