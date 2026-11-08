//! Temporary, NUL-terminated storage used only at the FFI boundary.
use crate::Result;
use smallvec::SmallVec;
use std::ffi::{CStr, CString, c_char};

#[derive(Debug, Clone, Copy)]
pub(crate) enum StringArg<'a> {
    Str(&'a str),
    CStr(&'a CStr),
}

/// Finish all writes before exposing any pointers: spilling the inline buffer
/// must never invalidate a pointer already placed in a native descriptor.
pub(crate) struct Strings {
    bytes: SmallVec<[u8; 1024]>,
}
impl Strings {
    pub(crate) fn new<'a>(values: impl IntoIterator<Item = StringArg<'a>>) -> Result<Self> {
        let mut bytes = SmallVec::new();
        for value in values {
            if let StringArg::Str(value) = value {
                if value.as_bytes().contains(&0) {
                    // Preserve the existing error type; allocate only on this error path.
                    return Err(CString::new(value).unwrap_err().into());
                }
                bytes.extend_from_slice(value.as_bytes());
                bytes.push(0);
            }
        }
        Ok(Self { bytes })
    }

    pub(crate) fn cursor(&self) -> StringCursor<'_> {
        StringCursor {
            bytes: &self.bytes,
            offset: 0,
        }
    }
}

pub(crate) struct StringCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl StringCursor<'_> {
    /// Inputs must be visited in the same order as in `Strings::new`.
    pub(crate) fn next(&mut self, value: StringArg<'_>) -> *const c_char {
        match value {
            StringArg::CStr(value) => value.as_ptr(),
            StringArg::Str(value) => {
                let end = self.offset + value.len() + 1;
                let bytes = &self.bytes[self.offset..end];
                self.offset = end;
                bytes.as_ptr().cast()
            }
        }
    }
}

pub(crate) fn with_cstr<T>(value: &str, call: impl FnOnce(&CStr) -> T) -> Result<T> {
    let strings = Strings::new([StringArg::Str(value)])?;
    // The buffer contains exactly one validated string followed by a NUL.
    let value = CStr::from_bytes_with_nul(&strings.bytes).expect("validated FFI string");
    Ok(call(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooled_strings_survive_spill_and_keep_cstrs_borrowed() {
        for len in [0, 1023, 4096] {
            let long = "x".repeat(len);
            let inputs = [
                StringArg::Str("first"),
                StringArg::CStr(c"borrowed"),
                StringArg::Str(&long),
                StringArg::Str(""),
            ];
            let strings = Strings::new(inputs).unwrap();
            assert_eq!(strings.bytes.spilled(), len > 1016);
            let mut cursor = strings.cursor();
            for input in inputs {
                let ptr = cursor.next(input);
                let expected = match input {
                    StringArg::Str(value) => value.as_bytes(),
                    StringArg::CStr(value) => {
                        assert_eq!(ptr, value.as_ptr());
                        value.to_bytes()
                    }
                };
                // SAFETY: all pointers refer to live, terminated storage above.
                assert_eq!(unsafe { CStr::from_ptr(ptr) }.to_bytes(), expected);
            }
        }
        assert!(matches!(
            Strings::new([StringArg::Str("bad\0value")]),
            Err(crate::Error::InteriorNul(_))
        ));
    }
}
