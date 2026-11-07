use crate::{Result, com::cstr, sys};
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
};

macro_rules! reflection {
    ($name:ident, $raw:ident) => {
        /// A borrowed reflection node. Its lifetime is bounded by its owning component.
        #[derive(Debug, Clone, Copy)]
        pub struct $name<'a> {
            ptr: NonNull<sys::$raw>,
            owner: PhantomData<&'a Rc<()>>,
        }
        impl<'a> $name<'a> {
            /// # Safety
            /// `ptr` must be null or a valid reflection node whose owning session and
            /// component remain alive and whose storage stays valid for all of `'a`.
            pub unsafe fn from_raw(ptr: *mut sys::$raw) -> Option<Self> {
                NonNull::new(ptr).map(|ptr| Self {
                    ptr,
                    owner: PhantomData,
                })
            }
            pub fn as_ptr(self) -> *mut sys::$raw {
                self.ptr.as_ptr()
            }
        }
    };
}
reflection!(ProgramLayout, slang_ShaderReflection);
reflection!(EntryPointLayout, slang_EntryPointReflection);
reflection!(VariableLayout, slang_VariableLayoutReflection);
reflection!(TypeLayout, slang_TypeLayoutReflection);
reflection!(TypeReflection, slang_TypeReflection);
reflection!(VariableReflection, slang_VariableReflection);

impl<'a> ProgramLayout<'a> {
    pub fn parameter_count(self) -> usize {
        unsafe { sys::slang_rs_slang_ShaderReflection_getParameterCount(self.as_ptr()) as usize }
    }
    pub fn parameter(self, index: usize) -> Option<VariableLayout<'a>> {
        if index >= self.parameter_count() {
            return None;
        }
        unsafe {
            VariableLayout::from_raw(sys::slang_rs_slang_ShaderReflection_getParameterByIndex(
                self.as_ptr(),
                index as u32,
            ))
        }
    }
    pub fn parameters(self) -> impl Iterator<Item = VariableLayout<'a>> {
        (0..self.parameter_count()).filter_map(move |i| self.parameter(i))
    }
    pub fn entry_point_count(self) -> usize {
        unsafe { sys::slang_rs_slang_ShaderReflection_getEntryPointCount(self.as_ptr()) as usize }
    }
    pub fn entry_point(self, index: usize) -> Option<EntryPointLayout<'a>> {
        if index >= self.entry_point_count() {
            return None;
        }
        unsafe {
            EntryPointLayout::from_raw(sys::slang_rs_slang_ShaderReflection_getEntryPointByIndex(
                self.as_ptr(),
                index as sys::SlangUInt,
            ))
        }
    }
    pub fn entry_points(self) -> impl Iterator<Item = EntryPointLayout<'a>> {
        (0..self.entry_point_count()).filter_map(move |i| self.entry_point(i))
    }
    pub fn find_type(self, name: &str) -> Result<Option<TypeReflection<'a>>> {
        let name = CString::new(name)?;
        Ok(unsafe {
            TypeReflection::from_raw(sys::slang_rs_slang_ShaderReflection_findTypeByName(
                self.as_ptr(),
                name.as_ptr(),
            ))
        })
    }
    pub fn global_constant_buffer_size(self) -> usize {
        unsafe { sys::slang_rs_slang_ShaderReflection_getGlobalConstantBufferSize(self.as_ptr()) }
    }
}
impl<'a> EntryPointLayout<'a> {
    pub fn name(self) -> Option<&'a CStr> {
        unsafe {
            cstr(sys::slang_rs_slang_EntryPointReflection_getName(
                self.as_ptr(),
            ))
        }
    }
    pub fn stage(self) -> sys::SlangStage {
        unsafe { sys::slang_rs_slang_EntryPointReflection_getStage(self.as_ptr()) }
    }
    pub fn thread_group_size(self) -> [sys::SlangUInt; 3] {
        let mut size = [0; 3];
        unsafe {
            sys::slang_rs_slang_EntryPointReflection_getComputeThreadGroupSize(
                self.as_ptr(),
                3,
                size.as_mut_ptr(),
            );
        }
        size
    }
    pub fn parameter_count(self) -> usize {
        unsafe {
            sys::slang_rs_slang_EntryPointReflection_getParameterCount(self.as_ptr()) as usize
        }
    }
    pub fn parameter(self, index: usize) -> Option<VariableLayout<'a>> {
        if index >= self.parameter_count() {
            return None;
        }
        unsafe {
            VariableLayout::from_raw(
                sys::slang_rs_slang_EntryPointReflection_getParameterByIndex(
                    self.as_ptr(),
                    index as u32,
                ),
            )
        }
    }
}
impl<'a> VariableLayout<'a> {
    pub fn name(self) -> Option<&'a CStr> {
        unsafe {
            cstr(sys::slang_rs_slang_VariableLayoutReflection_getName(
                self.as_ptr(),
            ))
        }
    }
    pub fn ty(self) -> Option<TypeLayout<'a>> {
        unsafe {
            TypeLayout::from_raw(sys::slang_rs_slang_VariableLayoutReflection_getTypeLayout(
                self.as_ptr(),
            ))
        }
    }
    pub fn variable(self) -> Option<VariableReflection<'a>> {
        unsafe {
            VariableReflection::from_raw(sys::slang_rs_slang_VariableLayoutReflection_getVariable(
                self.as_ptr(),
            ))
        }
    }
    pub fn binding_index(self) -> u32 {
        unsafe { sys::slang_rs_slang_VariableLayoutReflection_getBindingIndex(self.as_ptr()) }
    }
    pub fn binding_space(self) -> u32 {
        unsafe { sys::slang_rs_slang_VariableLayoutReflection_getBindingSpace(self.as_ptr()) }
    }
    pub fn offset(self, category: sys::SlangParameterCategory) -> usize {
        unsafe { sys::slang_rs_slang_VariableLayoutReflection_getOffset(self.as_ptr(), category) }
    }
}
impl<'a> TypeLayout<'a> {
    pub fn ty(self) -> Option<TypeReflection<'a>> {
        unsafe {
            TypeReflection::from_raw(sys::slang_rs_slang_TypeLayoutReflection_getType(
                self.as_ptr(),
            ))
        }
    }
    /// Slang's unknown/unbounded size sentinel values are preserved.
    pub fn size(self, category: sys::SlangParameterCategory) -> usize {
        unsafe { sys::slang_rs_slang_TypeLayoutReflection_getSize(self.as_ptr(), category) }
    }
    pub fn stride(self, category: sys::SlangParameterCategory) -> usize {
        unsafe { sys::slang_rs_slang_TypeLayoutReflection_getStride(self.as_ptr(), category) }
    }
    pub fn field_count(self) -> usize {
        unsafe { sys::slang_rs_slang_TypeLayoutReflection_getFieldCount(self.as_ptr()) as usize }
    }
    pub fn field(self, index: usize) -> Option<VariableLayout<'a>> {
        if index >= self.field_count() {
            return None;
        }
        unsafe {
            VariableLayout::from_raw(sys::slang_rs_slang_TypeLayoutReflection_getFieldByIndex(
                self.as_ptr(),
                index as u32,
            ))
        }
    }
    pub fn fields(self) -> impl Iterator<Item = VariableLayout<'a>> {
        (0..self.field_count()).filter_map(move |i| self.field(i))
    }
    pub fn element_type(self) -> Option<TypeLayout<'a>> {
        unsafe {
            TypeLayout::from_raw(
                sys::slang_rs_slang_TypeLayoutReflection_getElementTypeLayout(self.as_ptr()),
            )
        }
    }
    pub fn binding_range_count(self) -> usize {
        unsafe {
            sys::slang_rs_slang_TypeLayoutReflection_getBindingRangeCount(self.as_ptr()) as usize
        }
    }
}
impl<'a> TypeReflection<'a> {
    pub fn name(self) -> Option<&'a CStr> {
        unsafe { cstr(sys::slang_rs_slang_TypeReflection_getName(self.as_ptr())) }
    }
    pub fn kind(self) -> sys::slang_TypeReflection_Kind {
        unsafe { sys::slang_rs_slang_TypeReflection_getKind(self.as_ptr()) }
    }
    pub fn field_count(self) -> usize {
        unsafe { sys::slang_rs_slang_TypeReflection_getFieldCount(self.as_ptr()) as usize }
    }
    pub fn field(self, index: usize) -> Option<VariableReflection<'a>> {
        if index >= self.field_count() {
            return None;
        }
        unsafe {
            VariableReflection::from_raw(sys::slang_rs_slang_TypeReflection_getFieldByIndex(
                self.as_ptr(),
                index as u32,
            ))
        }
    }
    pub fn fields(self) -> impl Iterator<Item = VariableReflection<'a>> {
        (0..self.field_count()).filter_map(move |i| self.field(i))
    }
    pub fn element_type(self) -> Option<TypeReflection<'a>> {
        unsafe {
            TypeReflection::from_raw(sys::slang_rs_slang_TypeReflection_getElementType(
                self.as_ptr(),
            ))
        }
    }
}
impl<'a> VariableReflection<'a> {
    pub fn name(self) -> Option<&'a CStr> {
        unsafe {
            cstr(sys::slang_rs_slang_VariableReflection_getName(
                self.as_ptr(),
            ))
        }
    }
    pub fn ty(self) -> Option<TypeReflection<'a>> {
        unsafe {
            TypeReflection::from_raw(sys::slang_rs_slang_VariableReflection_getType(
                self.as_ptr(),
            ))
        }
    }
}
