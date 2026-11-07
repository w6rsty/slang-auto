impl Default for slang_CompilerOptionValue { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CompilerOptionValue_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_CompilerOptionEntry { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CompilerOptionEntry_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for SlangUUID { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_SlangUUID_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for ISlangUnknown {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangUnknown_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangUnknown_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangUnknown_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangUnknown_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangCastable {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangCastable_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangCastable_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangCastable_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangCastable_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangClonable {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangClonable_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangClonable_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangClonable_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangClonable_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangBlob {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangBlob_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangBlob_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangBlob_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangBlob_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for SlangTerminatedChars { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_SlangTerminatedChars_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for ISlangFileSystem {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangFileSystem_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangFileSystem_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangFileSystem_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangFileSystem_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangSharedLibrary_Dep1 {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangSharedLibrary_Dep1_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangSharedLibrary_Dep1_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangSharedLibrary_Dep1_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangSharedLibrary_Dep1_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangSharedLibrary {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangSharedLibrary_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangSharedLibrary_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangSharedLibrary_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangSharedLibrary_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangSharedLibraryLoader {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangSharedLibraryLoader_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangSharedLibraryLoader_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangSharedLibraryLoader_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangSharedLibraryLoader_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangFileSystemExt {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangFileSystemExt_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangFileSystemExt_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangFileSystemExt_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangFileSystemExt_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangMutableFileSystem {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangMutableFileSystem_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangMutableFileSystem_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangMutableFileSystem_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangMutableFileSystem_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangWriter {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangWriter_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangWriter_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangWriter_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangWriter_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for ISlangProfiler {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_ISlangProfiler_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_ISlangProfiler_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_ISlangProfiler_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_ISlangProfiler_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for SlangReflectionGenericArg { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_SlangReflectionGenericArg_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_ICompileRequest {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ICompileRequest_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ICompileRequest_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ICompileRequest_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ICompileRequest_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_GenericArgReflection { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_GenericArgReflection_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_DeclReflection_IteratedList { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_DeclReflection_IteratedList_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_DeclReflection_IteratedList_Iterator { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_DeclReflection_IteratedList_Iterator_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_IGlobalSession {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IGlobalSession_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IGlobalSession_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IGlobalSession_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IGlobalSession_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_TargetDesc { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_TargetDesc_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_PreprocessorMacroDesc { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_PreprocessorMacroDesc_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_SessionDesc { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_SessionDesc_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_SourceLocation { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_SourceLocation_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_ISession {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ISession_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ISession_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ISession_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ISession_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IMetadata {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IMetadata_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IMetadata_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IMetadata_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IMetadata_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IBindlessResourceMetadata {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IBindlessResourceMetadata_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IBindlessResourceMetadata_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IBindlessResourceMetadata_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IBindlessResourceMetadata_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_CoverageEntryInfo { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CoverageEntryInfo_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_CoverageBufferInfo { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CoverageBufferInfo_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_ICoverageTracingMetadata {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ICoverageTracingMetadata_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ICoverageTracingMetadata_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ICoverageTracingMetadata_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ICoverageTracingMetadata_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_SyntheticResourceInfo { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_SyntheticResourceInfo_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_ISyntheticResourceMetadata {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ISyntheticResourceMetadata_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ISyntheticResourceMetadata_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ISyntheticResourceMetadata_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ISyntheticResourceMetadata_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_CooperativeMatrixType { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CooperativeMatrixType_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_CooperativeMatrixCombination { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CooperativeMatrixCombination_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_CooperativeVectorTypeUsageInfo { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CooperativeVectorTypeUsageInfo_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_CooperativeVectorCombination { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_CooperativeVectorCombination_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_ICooperativeTypesMetadata {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ICooperativeTypesMetadata_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ICooperativeTypesMetadata_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ICooperativeTypesMetadata_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ICooperativeTypesMetadata_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_ICompileResult {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ICompileResult_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ICompileResult_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ICompileResult_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ICompileResult_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IComponentType {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IComponentType_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IComponentType_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IComponentType_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IComponentType_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IEntryPoint {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IEntryPoint_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IEntryPoint_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IEntryPoint_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IEntryPoint_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_ITypeConformance {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_ITypeConformance_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_ITypeConformance_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_ITypeConformance_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ITypeConformance_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IComponentType2 {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IComponentType2_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IComponentType2_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IComponentType2_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IComponentType2_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IModule {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IModule_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IModule_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IModule_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IModule_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}

unsafe impl Interface for slang_IModulePrecompileService_Experimental {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IModulePrecompileService_Experimental_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IModulePrecompileService_Experimental_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IModulePrecompileService_Experimental_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IModulePrecompileService_Experimental_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
impl Default for slang_SpecializationArg { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_SpecializationArg_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for SlangGlobalSessionDesc { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_SlangGlobalSessionDesc_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_VMExecOperand { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_VMExecOperand_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_VMExecInstHeader { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_VMExecInstHeader_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_ByteCodeFuncInfo { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ByteCodeFuncInfo_default(value.as_mut_ptr()); value.assume_init() } } }
impl Default for slang_ByteCodeRunnerDesc { fn default() -> Self { unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_ByteCodeRunnerDesc_default(value.as_mut_ptr()); value.assume_init() } } }

unsafe impl Interface for slang_IByteCodeRunner {
    unsafe fn add_ref(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_addRef(slang_rs_slang_IByteCodeRunner_as_unknown(ptr)); } }
    unsafe fn release(ptr: *mut Self) { unsafe { slang_rs_ISlangUnknown_release(slang_rs_slang_IByteCodeRunner_as_unknown(ptr)); } }
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {
        unsafe { slang_rs_ISlangUnknown_queryInterface(slang_rs_slang_IByteCodeRunner_as_unknown(ptr), guid, out) }
    }
    fn guid() -> SlangUUID {
        unsafe { let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_slang_IByteCodeRunner_getTypeGuid(value.as_mut_ptr()); value.assume_init() }
    }
}
