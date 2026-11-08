use crate::{Blob, ComPtr, Result, sys};
use std::{
    ffi::{CStr, c_char, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

impl Blob {
    /// Share immutable owned bytes with Slang without copying their contents.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self::from_storage(bytes.into())
    }

    /// Transfer an existing Vec allocation into a blob without repacking it.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self::from_storage(bytes)
    }

    fn from_storage<T: AsRef<[u8]> + Send + Sync + 'static>(bytes: T) -> Self {
        let bytes = Box::new(bytes);
        let (ptr, len) = (
            bytes.as_ref().as_ref().as_ptr(),
            bytes.as_ref().as_ref().len(),
        );
        unsafe extern "C" fn destroy<T>(context: *mut c_void) {
            // SAFETY: the adapter owns this Box until its final release.
            drop(unsafe { Box::from_raw(context.cast::<T>()) });
        }
        // SAFETY: owned storage remains stable and immutable until C++ releases it.
        Self(unsafe {
            ComPtr::from_raw_owned(sys::slang_auto_blob(
                ptr.cast(),
                len,
                Box::into_raw(bytes).cast(),
                destroy::<T>,
            ))
            .unwrap()
        })
    }
}

/// A thread-bound filesystem callback. Own its state and do not reenter the
/// session from the callback. Panics become failed file loads.
#[derive(Debug, Clone)]
pub struct FileSystem(pub(crate) ComPtr<sys::ISlangFileSystem>);

impl FileSystem {
    pub fn new(load: impl Fn(&CStr) -> Result<Option<Blob>> + 'static) -> Self {
        type Loader = Box<dyn Fn(&CStr) -> Result<Option<Blob>>>;
        unsafe extern "C" fn call(
            context: *mut c_void,
            path: *const c_char,
            out: *mut *mut sys::ISlangBlob,
        ) -> sys::SlangResult {
            let result = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: C++ supplies a live callback and terminated path.
                let load = unsafe { &*context.cast::<Loader>() };
                load(unsafe { CStr::from_ptr(path) })
            }));
            match result {
                Ok(Ok(Some(blob))) => {
                    // SAFETY: transfer one retained reference to the caller.
                    unsafe {
                        <sys::ISlangBlob as sys::Interface>::add_ref(blob.as_raw().as_ptr());
                        *out = blob.as_raw().as_ptr();
                    }
                    0
                }
                Ok(Ok(None)) => 1, // C++ maps this sentinel to SLANG_E_NOT_FOUND.
                _ => -1,
            }
        }
        unsafe extern "C" fn destroy(context: *mut c_void) {
            // SAFETY: the adapter exclusively owns this Box. A user destructor
            // must not unwind through C++.
            if catch_unwind(AssertUnwindSafe(|| {
                drop(unsafe { Box::from_raw(context.cast::<Loader>()) })
            }))
            .is_err()
            {
                std::process::abort();
            }
        }
        let loader: Box<Loader> = Box::new(Box::new(load));
        // SAFETY: the adapter takes ownership of loader and its callbacks.
        Self(unsafe {
            ComPtr::from_raw_owned(sys::slang_auto_filesystem(
                Box::into_raw(loader).cast(),
                call,
                destroy,
            ))
            .unwrap()
        })
    }
}
