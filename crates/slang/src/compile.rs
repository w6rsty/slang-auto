use crate::{
    Blob, Capability, ComPtr, Error, Output, ProgramLayout, Result, SessionDesc, TargetDesc,
    com::{cstr, diagnostics, owned_output},
    sys,
};
use smallvec::SmallVec;
use std::{
    ffi::{CStr, CString},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct GlobalSession(ComPtr<sys::slang_IGlobalSession>);
impl GlobalSession {
    pub fn new() -> Result<Self> {
        unsafe {
            owned_output("createGlobalSession", |out, _| {
                sys::slang_createGlobalSession2(&sys::SlangGlobalSessionDesc::default(), out)
            })
            .map(|o| Self(o.value))
        }
    }
    pub fn as_raw(&self) -> &ComPtr<sys::slang_IGlobalSession> {
        &self.0
    }
    pub fn find_profile(&self, name: &str) -> Result<sys::SlangProfileID> {
        crate::strings::with_cstr(name, |name| self.find_profile_cstr(name))?
    }
    /// Query a profile without allocating or copying the name.
    pub fn find_profile_cstr(&self, name: &CStr) -> Result<sys::SlangProfileID> {
        let profile = unsafe {
            sys::slang_rs_slang_IGlobalSession_findProfile(self.0.as_ptr(), name.as_ptr())
        };
        if profile == sys::SlangProfileID_SLANG_PROFILE_UNKNOWN {
            return Err(Error::InvalidArgument("Unknown Slang profile"));
        }
        Ok(profile)
    }
    pub fn find_capability(&self, name: &str) -> Result<Capability> {
        crate::strings::with_cstr(name, |name| self.find_capability_cstr(name))?
    }
    /// Query a capability without allocating or copying the name.
    pub fn find_capability_cstr(&self, name: &CStr) -> Result<Capability> {
        // SAFETY: the session and terminated name are live for the call.
        let capability = unsafe {
            sys::slang_rs_slang_IGlobalSession_findCapability(self.0.as_ptr(), name.as_ptr())
        };
        if capability == sys::SlangCapabilityID_SLANG_CAPABILITY_UNKNOWN {
            return Err(Error::InvalidArgument("Unknown Slang capability"));
        }
        Ok(Capability(capability))
    }
    pub fn create_session(&self, options: &SessionOptions) -> Result<Session> {
        let paths: SmallVec<[_; 8]> = options.search_paths.iter().map(String::as_str).collect();
        let defines: SmallVec<[_; 8]> = options
            .defines
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let targets: SmallVec<[_; 4]> = options
            .targets
            .iter()
            .map(|t| TargetDesc::new(t.format).profile(t.profile).flags(t.flags))
            .collect();
        self.create_session_desc(
            &SessionDesc::new()
                .targets(&targets)
                .search_paths(&paths)
                .defines(&defines)
                .matrix_layout_raw(options.matrix_layout),
        )
    }
    /// Full descriptor escape hatch, including future SDK options and file systems.
    ///
    /// # Safety
    /// Every pointer/count pair in `desc` must be valid for the SDK call. Any
    /// custom callbacks/file systems must meet Slang's ownership and thread rules.
    /// `target_count` must equal the nonnegative `desc.targetCount`.
    pub unsafe fn create_session_raw(
        &self,
        desc: &sys::slang_SessionDesc,
        target_count: usize,
    ) -> Result<Session> {
        unsafe {
            let raw = owned_output("createSession", |out, _| {
                sys::slang_rs_slang_IGlobalSession_createSession(self.0.as_ptr(), desc, out)
            })?
            .value
            .with_owner(self.clone());
            Ok(Session(Rc::new(SessionInner {
                raw,
                _global: self.clone(),
                target_count,
            })))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub format: sys::SlangCompileTarget,
    pub profile: sys::SlangProfileID,
    pub flags: sys::SlangTargetFlags,
}
impl Target {
    pub fn new(format: sys::SlangCompileTarget) -> Self {
        let d = sys::slang_TargetDesc::default();
        Self {
            format,
            profile: d.profile,
            flags: d.flags,
        }
    }
    pub fn spirv() -> Self {
        Self::new(sys::SlangCompileTarget_SLANG_SPIRV)
    }
    pub fn profile(mut self, profile: sys::SlangProfileID) -> Self {
        self.profile = profile;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SessionOptions {
    pub targets: Vec<Target>,
    pub search_paths: Vec<String>,
    pub defines: Vec<(String, String)>,
    pub matrix_layout: sys::SlangMatrixLayoutMode,
}
impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            targets: vec![Target::spirv()],
            search_paths: vec![],
            defines: vec![],
            matrix_layout: sys::slang_SessionDesc::default().defaultMatrixLayoutMode,
        }
    }
}

#[derive(Debug)]
struct SessionInner {
    raw: ComPtr<sys::slang_ISession>,
    _global: GlobalSession,
    target_count: usize,
}
#[derive(Debug, Clone)]
pub struct Session(Rc<SessionInner>);
impl Session {
    pub fn as_raw(&self) -> &ComPtr<sys::slang_ISession> {
        &self.0.raw
    }
    pub fn load_module(&self, name: &str) -> Result<Output<Module>> {
        let name = CString::new(name)?;
        unsafe {
            self.load(|diag| {
                sys::slang_rs_slang_ISession_loadModule(self.as_raw().as_ptr(), name.as_ptr(), diag)
            })
        }
    }
    pub fn load_module_from_source(
        &self,
        name: &str,
        path: &str,
        source: &str,
    ) -> Result<Output<Module>> {
        let (name, path, source) = (
            CString::new(name)?,
            CString::new(path)?,
            CString::new(source)?,
        );
        self.load_module_from_source_cstr(&name, &path, &source)
    }
    pub fn load_module_from_source_cstr(
        &self,
        name: &CStr,
        path: &CStr,
        source: &CStr,
    ) -> Result<Output<Module>> {
        unsafe {
            self.load(|diag| {
                sys::slang_rs_slang_ISession_loadModuleFromSourceString(
                    self.as_raw().as_ptr(),
                    name.as_ptr(),
                    path.as_ptr(),
                    source.as_ptr(),
                    diag,
                )
            })
        }
    }
    unsafe fn load(
        &self,
        call: impl FnOnce(*mut *mut sys::ISlangBlob) -> *mut sys::slang_IModule,
    ) -> Result<Output<Module>> {
        unsafe {
            let mut diag = std::ptr::null_mut();
            let ptr = call(&mut diag);
            let diagnostics = diagnostics(diag);
            // loadModule* returns a session-owned borrowed reference.
            let Some(raw) = ComPtr::from_raw_borrowed(ptr) else {
                return Err(Error::Slang {
                    code: -1,
                    diagnostics,
                });
            };
            Ok(Output {
                value: Module {
                    raw: raw.with_owner(self.clone()),
                    session: self.clone(),
                },
                diagnostics,
            })
        }
    }
    pub fn compose(&self, components: &[&Component]) -> Result<Output<Component>> {
        if components.is_empty() {
            return Err(Error::InvalidArgument(
                "Composition requires at least one component",
            ));
        }
        if components
            .iter()
            .any(|c| !Rc::ptr_eq(&self.0, &c.session.0))
        {
            return Err(Error::InvalidArgument(
                "All components must belong to the same session",
            ));
        }
        let pointers: Vec<_> = components.iter().map(|c| c.raw.as_ptr()).collect();
        let n = count(pointers.len())?;
        unsafe {
            owned_output("createCompositeComponentType", |out, diag| {
                sys::slang_rs_slang_ISession_createCompositeComponentType(
                    self.as_raw().as_ptr(),
                    pointers.as_ptr(),
                    n,
                    out,
                    diag,
                )
            })
        }
        .map(|o| {
            o.map(|raw| Component {
                raw: raw.with_owner((
                    self.clone(),
                    components.iter().map(|c| (*c).clone()).collect::<Vec<_>>(),
                )),
                session: self.clone(),
            })
        })
    }
    fn target(&self, index: usize) -> Result<sys::SlangInt> {
        if index >= self.0.target_count {
            return Err(Error::InvalidArgument("Target index out of bounds"));
        }
        count(index)
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    raw: ComPtr<sys::slang_IModule>,
    session: Session,
}
impl Module {
    pub fn as_raw(&self) -> &ComPtr<sys::slang_IModule> {
        &self.raw
    }
    pub fn as_component(&self) -> Component {
        let raw = unsafe {
            ComPtr::from_raw_borrowed(sys::slang_rs_slang_IModule_as_slang_IComponentType(
                self.raw.as_ptr(),
            ))
            .unwrap()
        };
        Component {
            raw: raw.with_owner(self.clone()),
            session: self.session.clone(),
        }
    }
    pub fn name(&self) -> Option<&CStr> {
        unsafe { cstr(sys::slang_rs_slang_IModule_getName(self.raw.as_ptr())) }
    }
    pub fn find_entry_point(&self, name: &str) -> Result<EntryPoint> {
        let name = CString::new(name)?;
        unsafe {
            owned_output("findEntryPointByName", |out, _| {
                sys::slang_rs_slang_IModule_findEntryPointByName(
                    self.raw.as_ptr(),
                    name.as_ptr(),
                    out,
                )
            })
        }
        .map(|o| EntryPoint {
            raw: o.value.with_owner(self.clone()),
            session: self.session.clone(),
        })
    }
    pub fn find_and_check_entry_point(
        &self,
        name: &str,
        stage: sys::SlangStage,
    ) -> Result<Output<EntryPoint>> {
        let name = CString::new(name)?;
        unsafe {
            owned_output("findAndCheckEntryPoint", |out, diag| {
                sys::slang_rs_slang_IModule_findAndCheckEntryPoint(
                    self.raw.as_ptr(),
                    name.as_ptr(),
                    stage,
                    out,
                    diag,
                )
            })
        }
        .map(|o| {
            o.map(|raw| EntryPoint {
                raw: raw.with_owner(self.clone()),
                session: self.session.clone(),
            })
        })
    }
    pub fn serialize(&self) -> Result<Blob> {
        unsafe {
            owned_output("serialize", |out, _| {
                sys::slang_rs_slang_IModule_serialize(self.raw.as_ptr(), out)
            })
        }
        .map(|o| Blob(o.value))
    }
}

#[derive(Debug, Clone)]
pub struct EntryPoint {
    raw: ComPtr<sys::slang_IEntryPoint>,
    session: Session,
}
impl EntryPoint {
    pub fn as_raw(&self) -> &ComPtr<sys::slang_IEntryPoint> {
        &self.raw
    }
    pub fn as_component(&self) -> Component {
        let raw = unsafe {
            ComPtr::from_raw_borrowed(sys::slang_rs_slang_IEntryPoint_as_slang_IComponentType(
                self.raw.as_ptr(),
            ))
            .unwrap()
        };
        Component {
            raw: raw.with_owner(self.clone()),
            session: self.session.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    raw: ComPtr<sys::slang_IComponentType>,
    session: Session,
}
impl Component {
    pub fn as_raw(&self) -> &ComPtr<sys::slang_IComponentType> {
        &self.raw
    }
    fn compile_result_interface(&self) -> Result<ComPtr<sys::slang_IComponentType2>> {
        self.raw.query().map_err(|error| match error {
            Error::Slang { code, .. } => Error::InterfaceQuery {
                interface: "IComponentType2",
                code,
            },
            other => other,
        })
    }
    /// Retrieve multiple outputs (for example code and separate debug data).
    /// The result retains its component; item roles/order are target-specific.
    pub fn target_compile_result(&self, target: usize) -> Result<Output<CompileResult>> {
        let target = self.session.target(target)?;
        let interface = self.compile_result_interface()?;
        // SAFETY: target is checked and the extension is live. Both outputs
        // transfer owned references; call_owned also retains the component chain.
        unsafe {
            interface.call_owned(|ptr, out, diag| {
                sys::slang_rs_slang_IComponentType2_getTargetCompileResult(ptr, target, out, diag)
            })
        }
        .map(|o| o.map(CompileResult))
    }
    /// Retrieve all outputs for a checked entry point and target.
    pub fn entry_point_compile_result(
        &self,
        entry: usize,
        target: usize,
    ) -> Result<Output<CompileResult>> {
        let target_index = self.session.target(target)?;
        let layout = self.layout(target)?;
        if entry >= layout.value.entry_point_count() {
            return Err(Error::InvalidArgument("Entry point index out of bounds"));
        }
        let entry = count(entry)?;
        let interface = self.compile_result_interface()?;
        // SAFETY: indices are checked, the interface is live, and outputs are
        // owned references as required by call_owned.
        let result = unsafe {
            interface.call_owned(|ptr, out, diag| {
                sys::slang_rs_slang_IComponentType2_getEntryPointCompileResult(
                    ptr,
                    entry,
                    target_index,
                    out,
                    diag,
                )
            })
        };
        match result {
            Ok(mut output) => {
                output.diagnostics.insert_str(0, &layout.diagnostics);
                Ok(output.map(CompileResult))
            }
            Err(Error::Slang {
                code,
                mut diagnostics,
            }) => {
                diagnostics.insert_str(0, &layout.diagnostics);
                Err(Error::Slang { code, diagnostics })
            }
            Err(error) => Err(error),
        }
    }
    pub fn link(&self) -> Result<Output<Self>> {
        unsafe {
            owned_output("link", |out, diag| {
                sys::slang_rs_slang_IComponentType_link(self.raw.as_ptr(), out, diag)
            })
        }
        .map(|o| {
            o.map(|raw| Self {
                raw: raw.with_owner(self.clone()),
                session: self.session.clone(),
            })
        })
    }
    pub fn layout(&self, target: usize) -> Result<Output<ProgramLayout<'_>>> {
        let target = self.session.target(target)?;
        unsafe {
            let mut diag = std::ptr::null_mut();
            let ptr =
                sys::slang_rs_slang_IComponentType_getLayout(self.raw.as_ptr(), target, &mut diag);
            let diagnostics = diagnostics(diag);
            let Some(value) = ProgramLayout::from_raw(ptr) else {
                return Err(Error::Slang {
                    code: -1,
                    diagnostics,
                });
            };
            Ok(Output { value, diagnostics })
        }
    }
    pub fn entry_point_code(&self, entry: usize, target: usize) -> Result<Output<Blob>> {
        let target_index = self.session.target(target)?;
        let layout = self.layout(target)?;
        if entry >= layout.value.entry_point_count() {
            return Err(Error::InvalidArgument("Entry point index out of bounds"));
        }
        let entry = count(entry)?;
        unsafe {
            owned_output("getEntryPointCode", |out, diag| {
                sys::slang_rs_slang_IComponentType_getEntryPointCode(
                    self.raw.as_ptr(),
                    entry,
                    target_index,
                    out,
                    diag,
                )
            })
        }
        .map(|mut o| {
            o.diagnostics.insert_str(0, &layout.diagnostics);
            o.map(Blob)
        })
    }
    pub fn target_code(&self, target: usize) -> Result<Output<Blob>> {
        let target = self.session.target(target)?;
        unsafe {
            owned_output("getTargetCode", |out, diag| {
                sys::slang_rs_slang_IComponentType_getTargetCode(
                    self.raw.as_ptr(),
                    target,
                    out,
                    diag,
                )
            })
        }
        .map(|o| o.map(Blob))
    }
    pub fn target_metadata(&self, target: usize) -> Result<Output<ComPtr<sys::slang_IMetadata>>> {
        let target = self.session.target(target)?;
        unsafe {
            owned_output("getTargetMetadata", |out, diag| {
                sys::slang_rs_slang_IComponentType_getTargetMetadata(
                    self.raw.as_ptr(),
                    target,
                    out,
                    diag,
                )
            })
        }
        .map(|o| o.map(|raw| raw.with_owner(self.clone())))
    }
}

/// An owned multi-blob compilation result. Does not assign universal meanings
/// to item indices: those depend on the target and enabled compiler options.
#[derive(Debug, Clone)]
pub struct CompileResult(ComPtr<sys::slang_ICompileResult>);
impl CompileResult {
    pub fn as_raw(&self) -> &ComPtr<sys::slang_ICompileResult> {
        &self.0
    }
    pub fn item_count(&self) -> usize {
        // SAFETY: the interface remains live throughout the call.
        unsafe { sys::slang_rs_slang_ICompileResult_getItemCount(self.0.as_ptr()) as usize }
    }
    /// Read an item without copying its bytes. Out-of-bounds indices return an
    /// error; use item_count to distinguish absent optional outputs.
    pub fn item(&self, index: usize) -> Result<Blob> {
        if index >= self.item_count() {
            return Err(Error::InvalidArgument(
                "Compile result item index out of bounds",
            ));
        }
        // SAFETY: index fits in the SDK's u32 item count. The SDK transfers an
        // owned immutable blob, and call_owned retains this result's dependencies.
        unsafe {
            self.0.call_owned(|ptr, out, _| {
                sys::slang_rs_slang_ICompileResult_getItemData(ptr, index as u32, out)
            })
        }
        .map(|output| Blob(output.value))
    }
    pub fn metadata(&self) -> Result<ComPtr<sys::slang_IMetadata>> {
        // SAFETY: getMetadata transfers an owned interface reference.
        unsafe {
            self.0
                .call_owned(|ptr, out, _| sys::slang_rs_slang_ICompileResult_getMetadata(ptr, out))
        }
        .map(|output| output.value)
    }
}

pub(crate) fn count(value: usize) -> Result<sys::SlangInt> {
    value
        .try_into()
        .map_err(|_| Error::InvalidArgument("Count exceeds SlangInt"))
}
