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

/// Returns the build tag of the loaded Slang compiler library without creating a session.
///
/// This identifies the runtime library, not this crate's version or the SDK headers
/// used to generate the bindings. Treat it as an opaque identifier: custom Slang
/// builds may use arbitrary text or a timestamp instead of a semantic version.
///
/// The library owns the NUL-terminated bytes for the process lifetime. No allocation
/// or UTF-8 conversion is performed; use [`std::ffi::CStr::to_str`] for checked UTF-8
/// or [`CStr::to_string_lossy`](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.to_string_lossy) for display.
///
/// ```
/// let tag = slang_auto::build_tag_string();
/// println!("Slang build: {}", tag.to_string_lossy());
/// ```
pub fn build_tag_string() -> &'static std::ffi::CStr {
    // SAFETY: spGetBuildTagString returns a non-null, NUL-terminated build tag.
    // Slang stores it in a string literal or a once-initialized static buffer,
    // independently of sessions. The linked library remains loaded for the
    // process lifetime; these bytes are neither mutated nor freed by callers.
    unsafe { std::ffi::CStr::from_ptr(sys::spGetBuildTagString()) }
}

// Keep the crates.io quickstart compiled by rustdoc instead of letting it drift.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_tests {}
