use crate::{
    Error, GlobalSession, Result, Session,
    strings::{StringArg, StringCursor, Strings},
    sys,
};
use smallvec::SmallVec;
use std::ffi::CStr;

macro_rules! sdk_enum {
    ($(#[$meta:meta])* $name:ident, $raw:ident, {$($variant:ident => $constant:ident),+ $(,)?}) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }
        impl From<$name> for sys::$raw {
            fn from(value: $name) -> Self {
                match value { $($name::$variant => sys::$constant),+ }
            }
        }
    };
}
sdk_enum!(OptimizationLevel, SlangOptimizationLevel, {
    None => SlangOptimizationLevel_SLANG_OPTIMIZATION_LEVEL_NONE,
    Default => SlangOptimizationLevel_SLANG_OPTIMIZATION_LEVEL_DEFAULT,
    High => SlangOptimizationLevel_SLANG_OPTIMIZATION_LEVEL_HIGH,
    Maximal => SlangOptimizationLevel_SLANG_OPTIMIZATION_LEVEL_MAXIMAL,
});
sdk_enum!(DebugInfoLevel, SlangDebugInfoLevel, {
    None => SlangDebugInfoLevel_SLANG_DEBUG_INFO_LEVEL_NONE,
    Minimal => SlangDebugInfoLevel_SLANG_DEBUG_INFO_LEVEL_MINIMAL,
    Standard => SlangDebugInfoLevel_SLANG_DEBUG_INFO_LEVEL_STANDARD,
    Maximal => SlangDebugInfoLevel_SLANG_DEBUG_INFO_LEVEL_MAXIMAL,
});
sdk_enum!(MatrixLayout, SlangMatrixLayoutMode, {
    RowMajor => SlangMatrixLayoutMode_SLANG_MATRIX_LAYOUT_ROW_MAJOR,
    ColumnMajor => SlangMatrixLayoutMode_SLANG_MATRIX_LAYOUT_COLUMN_MAJOR,
});
sdk_enum!(Stage, SlangStage, {
    Vertex => SlangStage_SLANG_STAGE_VERTEX,
    Fragment => SlangStage_SLANG_STAGE_FRAGMENT,
    Compute => SlangStage_SLANG_STAGE_COMPUTE,
    Geometry => SlangStage_SLANG_STAGE_GEOMETRY,
    Hull => SlangStage_SLANG_STAGE_HULL,
    Domain => SlangStage_SLANG_STAGE_DOMAIN,
    Mesh => SlangStage_SLANG_STAGE_MESH,
    Task => SlangStage_SLANG_STAGE_AMPLIFICATION,
    RayGeneration => SlangStage_SLANG_STAGE_RAY_GENERATION,
    Intersection => SlangStage_SLANG_STAGE_INTERSECTION,
    AnyHit => SlangStage_SLANG_STAGE_ANY_HIT,
    ClosestHit => SlangStage_SLANG_STAGE_CLOSEST_HIT,
    Miss => SlangStage_SLANG_STAGE_MISS,
    Callable => SlangStage_SLANG_STAGE_CALLABLE,
});

/// A capability recognized by the loaded Slang library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Capability(pub(crate) sys::SlangCapabilityID);
impl Capability {
    pub fn as_raw(self) -> sys::SlangCapabilityID {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
enum Value<'a> {
    Int(i32),
    Strings(StringArg<'a>, Option<StringArg<'a>>),
}

/// An allocation-free compiler option. Text is borrowed until session creation.
///
/// Constructors establish the SDK's name/value pairing. For unwrapped SDK
/// options, use the unsafe raw session descriptor escape hatch.
#[derive(Debug, Clone, Copy)]
pub struct CompilerOption<'a> {
    name: sys::slang_CompilerOptionName,
    value: Value<'a>,
}
impl<'a> CompilerOption<'a> {
    const fn int(name: sys::slang_CompilerOptionName, value: i32) -> Self {
        Self {
            name,
            value: Value::Int(value),
        }
    }
    pub const fn capability(value: Capability) -> Self {
        Self::int(sys::slang_CompilerOptionName_Capability, value.0)
    }
    pub fn optimization(value: OptimizationLevel) -> Self {
        Self::int(
            sys::slang_CompilerOptionName_Optimization,
            sys::SlangOptimizationLevel::from(value) as i32,
        )
    }
    pub fn debug_information(value: DebugInfoLevel) -> Self {
        Self::int(
            sys::slang_CompilerOptionName_DebugInformation,
            sys::SlangDebugInfoLevel::from(value) as i32,
        )
    }
    pub const fn emit_spirv_directly(value: bool) -> Self {
        Self::int(
            sys::slang_CompilerOptionName_EmitSpirvDirectly,
            value as i32,
        )
    }
    pub const fn emit_separate_debug(value: bool) -> Self {
        Self::int(
            sys::slang_CompilerOptionName_EmitSeparateDebug,
            value as i32,
        )
    }
    pub const fn vulkan_use_entry_point_name(value: bool) -> Self {
        Self::int(
            sys::slang_CompilerOptionName_VulkanUseEntryPointName,
            value as i32,
        )
    }
    pub const fn define(name: &'a str, value: &'a str) -> Self {
        Self {
            name: sys::slang_CompilerOptionName_MacroDefine,
            value: Value::Strings(StringArg::Str(name), Some(StringArg::Str(value))),
        }
    }
    /// Like `define`, without string validation or copying at the FFI boundary.
    pub const fn define_cstr(name: &'a CStr, value: &'a CStr) -> Self {
        Self {
            name: sys::slang_CompilerOptionName_MacroDefine,
            value: Value::Strings(StringArg::CStr(name), Some(StringArg::CStr(value))),
        }
    }
    pub const fn include_path(path: &'a str) -> Self {
        Self {
            name: sys::slang_CompilerOptionName_Include,
            value: Value::Strings(StringArg::Str(path), None),
        }
    }
    pub const fn include_path_cstr(path: &'a CStr) -> Self {
        Self {
            name: sys::slang_CompilerOptionName_Include,
            value: Value::Strings(StringArg::CStr(path), None),
        }
    }
    fn strings(self) -> impl Iterator<Item = StringArg<'a>> {
        match self.value {
            Value::Int(_) => [None, None],
            Value::Strings(first, second) => [Some(first), second],
        }
        .into_iter()
        .flatten()
    }
    fn raw(self, strings: &mut StringCursor<'_>) -> sys::slang_CompilerOptionEntry {
        let mut value = sys::slang_CompilerOptionValue::default();
        match self.value {
            Value::Int(integer) => {
                value.kind = sys::slang_CompilerOptionValueKind_Int;
                value.intValue0 = integer;
            }
            Value::Strings(first, second) => {
                value.kind = sys::slang_CompilerOptionValueKind_String;
                value.stringValue0 = strings.next(first);
                value.stringValue1 = second.map_or(std::ptr::null(), |s| strings.next(s));
            }
        }
        sys::slang_CompilerOptionEntry {
            name: self.name,
            value,
        }
    }
}

/// Borrowed code-generation target. Construction does not allocate.
#[derive(Debug, Clone, Copy)]
pub struct TargetDesc<'a> {
    format: sys::SlangCompileTarget,
    profile: sys::SlangProfileID,
    flags: sys::SlangTargetFlags,
    options: &'a [CompilerOption<'a>],
}
impl<'a> TargetDesc<'a> {
    pub fn new(format: sys::SlangCompileTarget) -> Self {
        let raw = sys::slang_TargetDesc::default();
        Self {
            format,
            profile: raw.profile,
            flags: raw.flags,
            options: &[],
        }
    }
    pub fn spirv() -> Self {
        Self::new(sys::SlangCompileTarget_SLANG_SPIRV)
    }
    pub fn spirv_asm() -> Self {
        Self::new(sys::SlangCompileTarget_SLANG_SPIRV_ASM)
    }
    pub fn profile(mut self, profile: sys::SlangProfileID) -> Self {
        self.profile = profile;
        self
    }
    pub fn flags(mut self, flags: sys::SlangTargetFlags) -> Self {
        self.flags = flags;
        self
    }
    pub fn compiler_options(mut self, options: &'a [CompilerOption<'a>]) -> Self {
        self.options = options;
        self
    }
}

/// Borrowed session configuration. Defaults follow the SDK, with no targets.
///
/// All strings accept ordinary Rust text; embedded NULs are rejected when the
/// session is created. Native arrays and terminated strings are prepared only
/// for that call. Common small configurations fit in inline scratch buffers;
/// larger ones spill to the heap. This does not remove Slang's own allocations
/// or the existing session ownership bookkeeping.
#[derive(Debug, Clone, Copy)]
pub struct SessionDesc<'a> {
    targets: &'a [TargetDesc<'a>],
    options: &'a [CompilerOption<'a>],
    paths: &'a [&'a str],
    defines: &'a [(&'a str, &'a str)],
    matrix_layout: sys::SlangMatrixLayoutMode,
}
impl Default for SessionDesc<'_> {
    fn default() -> Self {
        Self {
            targets: &[],
            options: &[],
            paths: &[],
            defines: &[],
            matrix_layout: sys::slang_SessionDesc::default().defaultMatrixLayoutMode,
        }
    }
}
impl<'a> SessionDesc<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn targets(mut self, targets: &'a [TargetDesc<'a>]) -> Self {
        self.targets = targets;
        self
    }
    pub fn compiler_options(mut self, options: &'a [CompilerOption<'a>]) -> Self {
        self.options = options;
        self
    }
    pub fn search_paths(mut self, paths: &'a [&'a str]) -> Self {
        self.paths = paths;
        self
    }
    pub fn defines(mut self, defines: &'a [(&'a str, &'a str)]) -> Self {
        self.defines = defines;
        self
    }
    pub fn matrix_layout(mut self, layout: MatrixLayout) -> Self {
        self.matrix_layout = layout.into();
        self
    }
    pub(crate) fn matrix_layout_raw(mut self, layout: sys::SlangMatrixLayoutMode) -> Self {
        self.matrix_layout = layout;
        self
    }

    // All native pointers stay scoped to this closure, after their storage has
    // finished growing. No self-referential Rust object escapes this function.
    fn with_raw<T>(&self, call: impl FnOnce(&sys::slang_SessionDesc) -> Result<T>) -> Result<T> {
        let all_options = || {
            self.options
                .iter()
                .chain(self.targets.iter().flat_map(|t| t.options.iter()))
        };
        let strings = Strings::new(
            self.paths
                .iter()
                .map(|s| StringArg::Str(s))
                .chain(
                    self.defines
                        .iter()
                        .flat_map(|(k, v)| [StringArg::Str(k), StringArg::Str(v)]),
                )
                .chain(all_options().flat_map(|o| o.strings())),
        )?;
        let mut cursor = strings.cursor();
        let paths: SmallVec<[_; 8]> = self
            .paths
            .iter()
            .map(|s| cursor.next(StringArg::Str(s)))
            .collect();
        let defines: SmallVec<[_; 8]> = self
            .defines
            .iter()
            .map(|(k, v)| sys::slang_PreprocessorMacroDesc {
                name: cursor.next(StringArg::Str(k)),
                value: cursor.next(StringArg::Str(v)),
            })
            .collect();
        let options: SmallVec<[_; 16]> = all_options().map(|o| o.raw(&mut cursor)).collect();
        let mut offset = self.options.len();
        let targets: SmallVec<[_; 4]> = self
            .targets
            .iter()
            .map(|target| {
                let start = offset;
                offset += target.options.len();
                Ok(sys::slang_TargetDesc {
                    format: target.format,
                    profile: target.profile,
                    flags: target.flags,
                    compilerOptionEntries: options[start..offset].as_ptr(),
                    compilerOptionEntryCount: count32(target.options.len())?,
                    ..Default::default()
                })
            })
            .collect::<Result<_>>()?;
        let raw = sys::slang_SessionDesc {
            targets: targets.as_ptr(),
            targetCount: crate::compile::count(targets.len())?,
            searchPaths: paths.as_ptr(),
            searchPathCount: crate::compile::count(paths.len())?,
            preprocessorMacros: defines.as_ptr(),
            preprocessorMacroCount: crate::compile::count(defines.len())?,
            compilerOptionEntries: options.as_ptr(),
            compilerOptionEntryCount: count32(self.options.len())?,
            defaultMatrixLayoutMode: self.matrix_layout,
            ..Default::default()
        };
        call(&raw)
    }
}
impl GlobalSession {
    /// Create a session from borrowed Rust configuration, with scoped FFI storage.
    pub fn create_session_desc(&self, desc: &SessionDesc<'_>) -> Result<Session> {
        desc.with_raw(|raw| {
            // SAFETY: all pointer/count pairs refer to the live buffers in
            // with_raw. Slang copies configuration during createSession. No
            // callbacks or borrowed file-system interfaces are supplied.
            unsafe { self.create_session_raw(raw, desc.targets.len()) }
        })
    }
}
fn count32(value: usize) -> Result<u32> {
    value
        .try_into()
        .map_err(|_| Error::InvalidArgument("Count exceeds u32"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };

    // Count Rust allocations only on the thread measuring descriptor preparation;
    // other tests and Slang's allocator do not affect this check.
    struct CountingAllocator;
    thread_local! {
        static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
    }
    fn record_allocation() {
        let _ = ALLOCATIONS.try_with(|count| {
            if let Some(value) = count.get() {
                count.set(Some(value + 1));
            }
        });
    }
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record_allocation();
            // SAFETY: forwarding the allocator contract unchanged to System.
            unsafe { System.alloc(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: forwarding the allocator contract unchanged to System.
            unsafe { System.dealloc(ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            record_allocation();
            // SAFETY: forwarding the allocator contract unchanged to System.
            unsafe { System.realloc(ptr, layout, size) }
        }
    }
    #[global_allocator]
    static ALLOCATOR: CountingAllocator = CountingAllocator;

    #[test]
    fn small_configuration_preparation_does_not_allocate() {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                ALLOCATIONS.with(|count| count.set(None));
            }
        }
        let _reset = Reset;
        ALLOCATIONS.with(|count| count.set(Some(0)));
        let options = [
            CompilerOption::define("FEATURE", "1"),
            CompilerOption::optimization(OptimizationLevel::High),
        ];
        let target_options = [CompilerOption::vulkan_use_entry_point_name(true)];
        let targets = [
            TargetDesc::spirv().compiler_options(&target_options),
            TargetDesc::spirv_asm(),
        ];
        let paths = ["shaders/include"];
        let defines = [("GROUP_SIZE", "8")];
        let desc = SessionDesc::new()
            .targets(&targets)
            .compiler_options(&options)
            .search_paths(&paths)
            .defines(&defines);
        desc.with_raw(|raw| {
            std::hint::black_box(raw);
            Ok(())
        })
        .unwrap();
        let count = ALLOCATIONS.with(|count| count.replace(None)).unwrap();
        assert_eq!(
            count, 0,
            "descriptor construction and FFI preparation must stay inline"
        );
    }

    #[test]
    fn native_arrays_remain_valid_after_all_scratch_buffers_spill() {
        let long = "x".repeat(2048);
        let paths = ["first/path", long.as_str()].repeat(8);
        let defines = [("DEFINE", long.as_str()); 12];
        let global_options = [CompilerOption::define("GLOBAL", &long); 20];
        let target_options = [
            CompilerOption::define_cstr(c"BORROWED", c"1"),
            CompilerOption::include_path(&long),
        ];
        let targets = [TargetDesc::spirv().compiler_options(&target_options); 6];
        SessionDesc::new()
            .targets(&targets)
            .search_paths(&paths)
            .defines(&defines)
            .compiler_options(&global_options)
            .with_raw(|raw| {
                assert_eq!(raw.targetCount, 6);
                assert_eq!(raw.searchPathCount, 16);
                assert_eq!(raw.preprocessorMacroCount, 12);
                assert_eq!(raw.compilerOptionEntryCount, 20);
                // SAFETY: with_raw keeps every native array and its strings live
                // throughout this closure; counts above match their lengths.
                unsafe {
                    let paths = std::slice::from_raw_parts(raw.searchPaths, 16);
                    for pair in paths.chunks_exact(2) {
                        assert_eq!(CStr::from_ptr(pair[0]).to_bytes(), b"first/path");
                        assert_eq!(CStr::from_ptr(pair[1]).to_bytes(), long.as_bytes());
                    }
                    for define in std::slice::from_raw_parts(raw.preprocessorMacros, 12) {
                        assert_eq!(CStr::from_ptr(define.name).to_bytes(), b"DEFINE");
                        assert_eq!(CStr::from_ptr(define.value).to_bytes(), long.as_bytes());
                    }
                    for option in std::slice::from_raw_parts(raw.compilerOptionEntries, 20) {
                        assert_eq!(
                            CStr::from_ptr(option.value.stringValue0).to_bytes(),
                            b"GLOBAL"
                        );
                        assert_eq!(
                            CStr::from_ptr(option.value.stringValue1).to_bytes(),
                            long.as_bytes()
                        );
                    }
                    for target in std::slice::from_raw_parts(raw.targets, 6) {
                        assert_eq!(target.compilerOptionEntryCount, 2);
                        let options = std::slice::from_raw_parts(target.compilerOptionEntries, 2);
                        assert_eq!(CStr::from_ptr(options[0].value.stringValue0), c"BORROWED");
                        assert_eq!(CStr::from_ptr(options[0].value.stringValue1), c"1");
                        assert_eq!(
                            CStr::from_ptr(options[1].value.stringValue0).to_bytes(),
                            long.as_bytes()
                        );
                        assert!(options[1].value.stringValue1.is_null());
                    }
                }
                Ok(())
            })
            .unwrap();
    }
}
