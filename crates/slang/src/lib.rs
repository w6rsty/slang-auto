//! Slang compilation and reflection with Rust ownership.
//!
//! [`sys`] contains every concrete public compiler API from the selected headers.
//! Use the safe layer for common operations and [`ComPtr`] for advanced interfaces.
//! Objects are intentionally neither `Send` nor `Sync`: use one session per thread.
//!
//! Reflection cannot outlive its component:
//! ```compile_fail
//! use slang_auto::{Component, ProgramLayout};
//! fn dangling(component: Component) -> ProgramLayout<'static> {
//!     component.layout(0).unwrap().value
//! }
//! ```
//! Sessions cannot be transferred across threads:
//! ```compile_fail
//! use slang_auto::GlobalSession;
//! let session = GlobalSession::new().unwrap();
//! std::thread::spawn(move || drop(session));
//! ```
pub use slang_auto_sys as sys;
mod com;
mod compile;
mod reflection;
pub use com::{Blob, ComPtr, Error, Output, Result};
pub use compile::{Component, EntryPoint, GlobalSession, Module, Session, SessionOptions, Target};
pub use reflection::*;

// Keep the crates.io quickstart compiled by rustdoc instead of letting it drift.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_tests {}
