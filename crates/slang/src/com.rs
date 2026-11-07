use crate::sys;
use std::{
    ffi::{CStr, NulError},
    fmt,
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InteriorNul(NulError),
    Slang {
        code: sys::SlangResult,
        diagnostics: String,
    },
    NullResult(&'static str),
    InvalidArgument(&'static str),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul(e) => write!(f, "FFI string contains an interior NUL: {e}"),
            Self::Slang { code, diagnostics } => {
                write!(f, "Slang error 0x{code:08x}: {diagnostics}")
            }
            Self::NullResult(op) => write!(f, "Slang returned no object from {op}"),
            Self::InvalidArgument(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for Error {}
impl From<NulError> for Error {
    fn from(e: NulError) -> Self {
        Self::InteriorNul(e)
    }
}

/// A result plus diagnostics, including warnings from successful operations.
#[derive(Debug)]
pub struct Output<T> {
    pub value: T,
    pub diagnostics: String,
}
impl<T> Output<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Output<U> {
        Output {
            value: f(self.value),
            diagnostics: self.diagnostics,
        }
    }
}

/// Owns exactly one reference to a Slang COM interface.
/// `Clone` calls `addRef`; `Drop` calls `release` through the generated C++ bridge.
pub struct ComPtr<T: sys::Interface> {
    ptr: NonNull<T>,
    thread_bound: PhantomData<Rc<()>>,
    // Retain context even when a safe wrapper's as_raw() handle is cloned or queried.
    keep_alive: Option<Rc<dyn std::any::Any>>,
}
impl<T: sys::Interface> fmt::Debug for ComPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ComPtr").field(&self.ptr).finish()
    }
}
impl<T: sys::Interface> ComPtr<T> {
    /// Adopt an existing reference, without calling `addRef`.
    ///
    /// # Safety
    /// A non-null pointer must be a live `T` with one owned reference transferred to
    /// this object. All dependencies must remain alive, and callers must uphold
    /// the interface's thread and aliasing requirements for this object's lifetime.
    pub unsafe fn from_raw_owned(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            thread_bound: PhantomData,
            keep_alive: None,
        })
    }
    /// Retain a borrowed interface pointer.
    ///
    /// # Safety
    /// The pointer must be null or a live `T`. The dependencies, thread and aliasing
    /// requirements of [`Self::from_raw_owned`] also apply.
    pub unsafe fn from_raw_borrowed(ptr: *mut T) -> Option<Self> {
        unsafe {
            let value = Self::from_raw_owned(ptr)?;
            T::add_ref(ptr);
            Some(value)
        }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
    pub(crate) fn with_owner(mut self, owner: impl std::any::Any) -> Self {
        self.keep_alive = Some(Rc::new(owner));
        self
    }
    /// Query an extension interface using its SDK-generated UUID.
    pub fn query<U: sys::Interface>(&self) -> Result<ComPtr<U>> {
        unsafe {
            let mut out = std::ptr::null_mut();
            let code = T::query_interface(self.as_ptr(), &U::guid(), &mut out);
            let value = ComPtr::from_raw_owned(out.cast());
            check(code, String::new())?;
            value
                .map(|v| v.with_owner(self.clone()))
                .ok_or(Error::NullResult("queryInterface"))
        }
    }

    /// Invoke a generated API with an owned interface output and optional diagnostics.
    /// The returned object retains this handle and its dependencies.
    /// This supports new SDK methods without adding a handwritten high-level wrapper.
    ///
    /// # Safety
    /// The closure must obey the called Slang API's argument and synchronization
    /// contracts. Any non-null output must transfer one owned reference of `U`,
    /// and any diagnostic output must transfer one owned immutable blob reference.
    /// A successful call must return a non-null interface. The returned object
    /// must remain valid while its reference and this source handle are retained.
    pub unsafe fn call_owned<U: sys::Interface>(
        &self,
        call: impl FnOnce(*mut T, *mut *mut U, *mut *mut sys::ISlangBlob) -> sys::SlangResult,
    ) -> Result<Output<ComPtr<U>>> {
        unsafe {
            owned_output("generated interface call", |out, diag| {
                call(self.as_ptr(), out, diag)
            })
            .map(|o| o.map(|value| value.with_owner(self.clone())))
        }
    }
}
impl<T: sys::Interface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            T::add_ref(self.as_ptr());
            Self {
                ptr: self.ptr,
                thread_bound: PhantomData,
                keep_alive: self.keep_alive.clone(),
            }
        }
    }
}
impl<T: sys::Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            T::release(self.as_ptr());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Blob(pub(crate) ComPtr<sys::ISlangBlob>);
impl Blob {
    /// # Safety
    /// `raw` must own a valid immutable Slang blob whose backing storage remains
    /// valid for the lifetime of the blob reference.
    pub unsafe fn from_com(raw: ComPtr<sys::ISlangBlob>) -> Self {
        Self(raw)
    }
    pub fn as_raw(&self) -> &ComPtr<sys::ISlangBlob> {
        &self.0
    }
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let size = sys::slang_rs_ISlangBlob_getBufferSize(self.0.as_ptr());
            if size == 0 {
                return &[];
            }
            let ptr = sys::slang_rs_ISlangBlob_getBufferPointer(self.0.as_ptr()).cast::<u8>();
            assert!(
                !ptr.is_null() && size <= isize::MAX as usize,
                "Invalid Slang blob"
            );
            std::slice::from_raw_parts(ptr, size)
        }
    }
    /// Text blobs sometimes include a terminal NUL. Binary access never removes it.
    pub fn as_str(&self) -> std::result::Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.text_bytes())
    }
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(self.text_bytes())
    }
    fn text_bytes(&self) -> &[u8] {
        let b = self.as_bytes();
        b.strip_suffix(&[0]).unwrap_or(b)
    }
}

pub(crate) fn check(code: sys::SlangResult, diagnostics: String) -> Result<()> {
    if code < 0 {
        Err(Error::Slang { code, diagnostics })
    } else {
        Ok(())
    }
}
pub(crate) unsafe fn diagnostics(ptr: *mut sys::ISlangBlob) -> String {
    unsafe {
        ComPtr::from_raw_owned(ptr)
            .map(|p| Blob(p).to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}
pub(crate) unsafe fn owned_output<T: sys::Interface>(
    operation: &'static str,
    call: impl FnOnce(*mut *mut T, *mut *mut sys::ISlangBlob) -> sys::SlangResult,
) -> Result<Output<ComPtr<T>>> {
    unsafe {
        let mut ptr = std::ptr::null_mut();
        let mut diag = std::ptr::null_mut();
        let code = call(&mut ptr, &mut diag);
        let value = ComPtr::from_raw_owned(ptr); // Also release any output on failure.
        let diagnostics = diagnostics(diag);
        if code < 0 {
            return Err(Error::Slang { code, diagnostics });
        }
        Ok(Output {
            value: value.ok_or(Error::NullResult(operation))?,
            diagnostics,
        })
    }
}
pub(crate) unsafe fn cstr<'a>(ptr: *const std::ffi::c_char) -> Option<&'a CStr> {
    unsafe {
        if ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ptr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, ffi::c_void};

    #[derive(Default)]
    struct Counts {
        references: Cell<usize>,
        destroyed: Cell<bool>,
    }

    // A Rust implementation of the abstract ownership contract; no fake C++ vtable.
    struct MockInterface(Rc<Counts>);
    unsafe impl sys::Interface for MockInterface {
        unsafe fn add_ref(ptr: *mut Self) {
            let counts = unsafe { &(*ptr).0 };
            counts.references.set(counts.references.get() + 1);
        }
        unsafe fn release(ptr: *mut Self) {
            let counts = unsafe { &(*ptr).0 };
            counts.references.set(counts.references.get() - 1);
            if counts.references.get() == 0 {
                counts.destroyed.set(true);
                drop(unsafe { Box::from_raw(ptr) });
            }
        }
        unsafe fn query_interface(
            ptr: *mut Self,
            _: &sys::SlangUUID,
            out: *mut *mut c_void,
        ) -> sys::SlangResult {
            unsafe {
                Self::add_ref(ptr);
                *out = ptr.cast();
            }
            0
        }
        fn guid() -> sys::SlangUUID {
            sys::SlangUUID {
                data1: 0,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            }
        }
    }

    fn mock() -> (ComPtr<MockInterface>, Rc<Counts>) {
        let counts = Rc::new(Counts::default());
        counts.references.set(1);
        let ptr = Box::into_raw(Box::new(MockInterface(counts.clone())));
        (unsafe { ComPtr::from_raw_owned(ptr).unwrap() }, counts)
    }

    #[test]
    fn owned_clone_and_drop_balance_references() {
        let (owner, counts) = mock();
        let clone = owner.clone();
        assert_eq!(counts.references.get(), 2);
        drop(owner);
        assert!(!counts.destroyed.get());
        drop(clone);
        assert_eq!(counts.references.get(), 0);
        assert!(counts.destroyed.get());
    }

    #[test]
    fn borrowed_pointer_retains_one_reference() {
        let (owner, counts) = mock();
        let borrowed = unsafe { ComPtr::from_raw_borrowed(owner.as_ptr()).unwrap() };
        assert_eq!(counts.references.get(), 2);
        drop(owner);
        assert!(!counts.destroyed.get());
        drop(borrowed);
        assert!(counts.destroyed.get());
    }

    #[test]
    fn null_pointer_never_creates_a_handle() {
        unsafe {
            assert!(ComPtr::<MockInterface>::from_raw_owned(std::ptr::null_mut()).is_none());
            assert!(ComPtr::<MockInterface>::from_raw_borrowed(std::ptr::null_mut()).is_none());
        }
    }

    #[test]
    fn failed_call_releases_partial_output() {
        let (owner, counts) = mock();
        let result = unsafe {
            owner.call_owned(|ptr, out, _| {
                <MockInterface as sys::Interface>::add_ref(ptr);
                *out = ptr;
                -1
            })
        };
        assert!(matches!(result, Err(Error::Slang { code: -1, .. })));
        assert_eq!(counts.references.get(), 1);
        drop(owner);
        assert!(counts.destroyed.get());
    }

    #[test]
    fn successful_call_requires_nonnull_output() {
        let (owner, _) = mock();
        let result = unsafe { owner.call_owned::<MockInterface>(|_, _, _| 0) };
        assert!(matches!(result, Err(Error::NullResult(_))));
    }

    #[test]
    fn queried_interface_retains_parent_dependency() {
        let (owner, counts) = mock();
        let context = Rc::new(());
        let weak_context = Rc::downgrade(&context);
        let owner = owner.with_owner(context);
        let queried = owner.query::<MockInterface>().unwrap();
        drop(owner);
        assert!(weak_context.upgrade().is_some());
        assert!(!counts.destroyed.get());
        drop(queried);
        assert!(weak_context.upgrade().is_none());
        assert!(counts.destroyed.get());
    }

    #[test]
    fn owned_call_retains_parent_until_child_is_released() {
        let (parent, parent_counts) = mock();
        let child_counts = Rc::new(Counts::default());
        child_counts.references.set(1);
        let child_ptr = Box::into_raw(Box::new(MockInterface(child_counts.clone())));
        let child = unsafe {
            parent.call_owned(|_, out, _| {
                *out = child_ptr;
                1
            })
        }
        .unwrap()
        .value;
        drop(parent);
        assert!(!parent_counts.destroyed.get());
        drop(child);
        assert!(child_counts.destroyed.get());
        assert!(parent_counts.destroyed.get());
    }
}
