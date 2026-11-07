//! Complete raw compiler API, generated from the selected SDK at build time.
//! All pointer validity, ownership and synchronization requirements are the caller's responsibility.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::all, improper_ctypes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Implemented automatically for every interface derived from `ISlangUnknown`.
///
/// # Safety
/// Implementations must use the actual C++ interface's reference counting and IID.
pub unsafe trait Interface: Sized + 'static {
    unsafe fn add_ref(ptr: *mut Self);
    unsafe fn release(ptr: *mut Self);
    unsafe fn query_interface(
        ptr: *mut Self,
        guid: &SlangUUID,
        out: *mut *mut ::std::ffi::c_void,
    ) -> SlangResult;
    fn guid() -> SlangUUID;
}
include!(concat!(env!("OUT_DIR"), "/interfaces.rs"));

/// Auditable list of generated methods, functions, casts and default initializers.
pub const API_MANIFEST: &str = include_str!(concat!(env!("OUT_DIR"), "/api-manifest.txt"));
