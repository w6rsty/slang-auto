use crate::{
    ComPtr, Error, ParameterCategory, Result, ScalarType, Stage, TypeKind, com::cstr, sys,
};
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
    /// Finds a global parameter by its reflected name.
    pub fn parameter_named(self, name: &str) -> Option<VariableLayout<'a>> {
        self.parameters()
            .find(|parameter| named(parameter.name(), name))
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
    /// Finds an entry point by its reflected name.
    pub fn entry_point_named(self, name: &str) -> Option<EntryPointLayout<'a>> {
        self.entry_points().find(|entry| named(entry.name(), name))
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
    /// Returns the shader stage if it is recognized by the Rust wrapper.
    pub fn shader_stage(self) -> Option<Stage> {
        Stage::from_raw(self.stage())
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
    pub fn parameters(self) -> impl Iterator<Item = VariableLayout<'a>> {
        (0..self.parameter_count()).filter_map(move |i| self.parameter(i))
    }
    pub fn parameter_named(self, name: &str) -> Option<VariableLayout<'a>> {
        self.parameters()
            .find(|parameter| named(parameter.name(), name))
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
    /// Returns the relative register space or descriptor set for the given category.
    pub fn binding_space_for(self, category: impl Into<ParameterCategory>) -> usize {
        unsafe {
            sys::spReflectionVariableLayout_GetSpace(self.as_ptr().cast(), category.into().as_raw())
        }
    }
    /// Returns the relative offset in the given layout unit.
    pub fn offset(self, category: impl Into<ParameterCategory>) -> usize {
        unsafe {
            sys::slang_rs_slang_VariableLayoutReflection_getOffset(
                self.as_ptr(),
                category.into().as_raw(),
            )
        }
    }
    /// Returns the relative byte offset within the enclosing uniform layout.
    pub fn uniform_offset(self) -> usize {
        self.offset(ParameterCategory::Uniform)
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
    /// Returns the reflected kind of this type.
    pub fn type_kind(self) -> TypeKind {
        self.ty().map_or(TypeKind::None, TypeReflection::type_kind)
    }
    /// Returns Slang's primary category for this layout.
    pub fn parameter_category(self) -> ParameterCategory {
        unsafe { sys::slang_rs_slang_TypeLayoutReflection_getParameterCategory(self.as_ptr()) }
            .into()
    }
    pub fn category_count(self) -> usize {
        unsafe { sys::slang_rs_slang_TypeLayoutReflection_getCategoryCount(self.as_ptr()) as usize }
    }
    /// Returns a layout category by index.
    pub fn category(self, index: usize) -> Option<ParameterCategory> {
        if index >= self.category_count() {
            return None;
        }
        Some(
            unsafe {
                sys::slang_rs_slang_TypeLayoutReflection_getCategoryByIndex(
                    self.as_ptr(),
                    index as u32,
                )
            }
            .into(),
        )
    }
    /// Iterates over the units in which this layout occupies space.
    pub fn categories(self) -> impl Iterator<Item = ParameterCategory> {
        (0..self.category_count()).filter_map(move |i| self.category(i))
    }
    /// Slang's unknown/unbounded size sentinel values are preserved.
    pub fn size(self, category: impl Into<ParameterCategory>) -> usize {
        unsafe {
            sys::slang_rs_slang_TypeLayoutReflection_getSize(
                self.as_ptr(),
                category.into().as_raw(),
            )
        }
    }
    /// Returns the stride in the given unit, preserving Slang's sentinel values.
    pub fn stride(self, category: impl Into<ParameterCategory>) -> usize {
        unsafe {
            sys::slang_rs_slang_TypeLayoutReflection_getStride(
                self.as_ptr(),
                category.into().as_raw(),
            )
        }
    }
    /// Returns the alignment in the given unit when Slang reports a positive value.
    pub fn alignment(self, category: impl Into<ParameterCategory>) -> Option<u32> {
        let value = unsafe {
            sys::slang_rs_slang_TypeLayoutReflection_getAlignment(
                self.as_ptr(),
                category.into().as_raw(),
            )
        };
        u32::try_from(value).ok().filter(|&value| value > 0)
    }
    /// Returns the size of the uniform data in bytes.
    pub fn uniform_size(self) -> usize {
        self.size(ParameterCategory::Uniform)
    }
    /// Returns the uniform stride in bytes.
    pub fn uniform_stride(self) -> usize {
        self.stride(ParameterCategory::Uniform)
    }
    /// Returns the uniform alignment in bytes when Slang reports a positive value.
    pub fn uniform_alignment(self) -> Option<u32> {
        self.alignment(ParameterCategory::Uniform)
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
    /// Finds a field in this layout by its reflected name.
    pub fn field_named(self, name: &str) -> Option<VariableLayout<'a>> {
        self.fields().find(|field| named(field.name(), name))
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
    /// Returns the reflected type kind.
    pub fn type_kind(self) -> TypeKind {
        TypeKind::from_raw(self.kind() as sys::SlangTypeKind)
    }
    /// Returns the scalar component reported by Slang.
    pub fn scalar_type(self) -> ScalarType {
        unsafe { sys::slang_rs_slang_TypeReflection_getScalarType(self.as_ptr()) }.into()
    }
    pub fn row_count(self) -> u32 {
        unsafe { sys::slang_rs_slang_TypeReflection_getRowCount(self.as_ptr()) }
    }
    pub fn column_count(self) -> u32 {
        unsafe { sys::slang_rs_slang_TypeReflection_getColumnCount(self.as_ptr()) }
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
    pub fn field_named(self, name: &str) -> Option<VariableReflection<'a>> {
        self.fields().find(|field| named(field.name(), name))
    }
    pub fn element_type(self) -> Option<TypeReflection<'a>> {
        unsafe {
            TypeReflection::from_raw(sys::slang_rs_slang_TypeReflection_getElementType(
                self.as_ptr(),
            ))
        }
    }
}

impl ComPtr<sys::slang_IMetadata> {
    /// Reports whether the compiled program uses this parameter location.
    pub fn is_parameter_location_used(
        &self,
        category: impl Into<ParameterCategory>,
        space: usize,
        index: usize,
    ) -> Result<bool> {
        let mut used = false;
        let code = unsafe {
            sys::slang_rs_slang_IMetadata_isParameterLocationUsed(
                self.as_ptr(),
                category.into().as_raw(),
                space as sys::SlangUInt,
                index as sys::SlangUInt,
                &mut used,
            )
        };
        if code < 0 {
            return Err(Error::Slang {
                code,
                diagnostics: String::new(),
            });
        }
        Ok(used)
    }
}

fn named(name: Option<&CStr>, expected: &str) -> bool {
    name.is_some_and(|name| name.to_bytes() == expected.as_bytes())
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
