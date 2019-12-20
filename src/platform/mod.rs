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

mod eventfd;
mod subreaper;
mod unix_common;

pub use eventfd::*;
pub use subreaper::*;
pub use unix_common::*;
