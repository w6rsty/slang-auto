use crate::{
    Blob, Capability, ComPtr, Error, Output, ProgramLayout, Result, SessionDesc, TargetDesc,
    com::{cstr, diagnostics, owned_output},
    sys,
};
use smallvec::SmallVec;
use std::{
    ffi::{CStr, CString},
    ptr::NonNull,
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
    /// Compose owned components. Existing callers can keep using this entry point.
    /// For modules and entry points without temporary owned conversions, use
    /// [`Self::compose_refs`]. Both paths share validation and inline buffers.
    pub fn compose(&self, components: &[&Component]) -> Result<Output<Component>> {
        self.compose_impl(components.iter().map(|c| c.as_component_ref()))
    }
    /// Compose borrowed module, entry-point, or component views.
    ///
    /// Up to four inputs fit in inline pointer and retained-handle buffers.
    /// Larger lists spill to the heap. On success, the result retains one COM
    /// reference per input and shares each input's existing dependency graph.
    /// Inputs may then be dropped; the result has no borrow of this slice.
    pub fn compose_refs(&self, components: &[ComponentRef<'_>]) -> Result<Output<Component>> {
        self.compose_impl(components.iter().copied())
    }
    fn compose_impl<'a>(
        &self,
        components: impl ExactSizeIterator<Item = ComponentRef<'a>> + Clone,
    ) -> Result<Output<Component>> {
        let pointers = self.component_pointers(components.clone())?;
        let n = count(pointers.len())?;
        // SAFETY: every view borrows a live source in this session; pointers were
        // converted through the generated C++ bridge, including base adjustments.
        let output = unsafe {
            owned_output("createCompositeComponentType", |out, diag| {
                sys::slang_rs_slang_ISession_createCompositeComponentType(
                    self.as_raw().as_ptr(),
                    pointers.as_ptr(),
                    n,
                    out,
                    diag,
                )
            })
        }?;
        // Only retain inputs after success. Sharing their ComPtr dependency
        // graphs preserves raw-handle escape semantics without allocating an
        // owned intermediate Component for each module or entry point.
        let retained: SmallVec<[_; 4]> = components.map(ComponentRef::retain).collect();
        Ok(output.map(|raw| Component {
            raw: raw.with_owner((self.clone(), retained)),
            session: self.clone(),
        }))
    }
    fn component_pointers<'a>(
        &self,
        components: impl ExactSizeIterator<Item = ComponentRef<'a>> + Clone,
    ) -> Result<SmallVec<[*mut sys::slang_IComponentType; 4]>> {
        if components.len() == 0 {
            return Err(Error::InvalidArgument(
                "Composition requires at least one component",
            ));
        }
        if components
            .clone()
            .any(|c| !Rc::ptr_eq(&self.0, &c.session().0))
        {
            return Err(Error::InvalidArgument(
                "All components must belong to the same session",
            ));
        }
        count(components.len())?;
        Ok(components.map(ComponentRef::as_ptr).collect())
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
    /// Borrow a component view without allocating or changing reference counts.
    #[inline]
    pub fn as_component_ref(&self) -> ComponentRef<'_> {
        // SAFETY: this is a live module; the generated bridge adjusts the base
        // pointer correctly. The returned view borrows this module.
        let ptr = unsafe { sys::slang_rs_slang_IModule_as_slang_IComponentType(self.raw.as_ptr()) };
        ComponentRef {
            ptr: NonNull::new(ptr).expect("module base interface"),
            source: ComponentSource::Module(self),
        }
    }
    /// Create an owned component reference. For a temporary composition input,
    /// prefer [`Self::as_component_ref`], which does not retain or allocate.
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
    /// Borrow a component view without allocating or changing reference counts.
    #[inline]
    pub fn as_component_ref(&self) -> ComponentRef<'_> {
        // SAFETY: this is a live entry point and the bridge performs the C++
        // base conversion. The returned view borrows this entry point.
        let ptr =
            unsafe { sys::slang_rs_slang_IEntryPoint_as_slang_IComponentType(self.raw.as_ptr()) };
        ComponentRef {
            ptr: NonNull::new(ptr).expect("entry-point base interface"),
            source: ComponentSource::EntryPoint(self),
        }
    }
    /// Create an owned component reference. For temporary composition inputs,
    /// prefer [`Self::as_component_ref`].
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

#[derive(Debug, Clone, Copy)]
enum ComponentSource<'a> {
    Module(&'a Module),
    EntryPoint(&'a EntryPoint),
    Component(&'a Component),
}

/// A borrowed component view for [`Session::compose_refs`]. Creating, copying,
/// and dropping a view neither allocates nor changes COM reference counts.
/// It cannot outlive its source and remains neither `Send` nor `Sync`.
///
/// ```compile_fail
/// use slang_auto::{ComponentRef, Module};
/// fn dangling(module: Module) -> ComponentRef<'static> {
///     module.as_component_ref()
/// }
/// ```
/// ```compile_fail
/// use slang_auto::{ComponentRef, EntryPoint};
/// fn dangling(entry: EntryPoint) -> ComponentRef<'static> {
///     entry.as_component_ref()
/// }
/// ```
/// ```compile_fail
/// use slang_auto::{Component, ComponentRef};
/// fn dangling(component: Component) -> ComponentRef<'static> {
///     component.as_component_ref()
/// }
/// ```
/// ```compile_fail
/// use slang_auto::ComponentRef;
/// fn require_send<T: Send>() {}
/// require_send::<ComponentRef<'static>>();
/// ```
/// ```compile_fail
/// use slang_auto::ComponentRef;
/// fn require_sync<T: Sync>() {}
/// require_sync::<ComponentRef<'static>>();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ComponentRef<'a> {
    ptr: NonNull<sys::slang_IComponentType>,
    source: ComponentSource<'a>,
}
impl<'a> ComponentRef<'a> {
    /// Borrowed raw interface pointer. Calling raw methods is unsafe; the
    /// pointer is only valid while the source and its dependencies remain live.
    #[inline]
    pub fn as_ptr(self) -> *mut sys::slang_IComponentType {
        self.ptr.as_ptr()
    }
    fn session(self) -> &'a Session {
        match self.source {
            ComponentSource::Module(source) => &source.session,
            ComponentSource::EntryPoint(source) => &source.session,
            ComponentSource::Component(source) => &source.session,
        }
    }
    fn retain(self) -> ComPtr<sys::slang_IComponentType> {
        // SAFETY: private constructors obtain ptr via the generated base cast
        // (or directly for Component). Each source raw handle already retains
        // its dependencies; the composition also retains the common session.
        unsafe {
            match self.source {
                ComponentSource::Module(source) => source.raw.retain_interface(self.as_ptr()),
                ComponentSource::EntryPoint(source) => source.raw.retain_interface(self.as_ptr()),
                ComponentSource::Component(source) => source.raw.retain_interface(self.as_ptr()),
            }
        }
    }
}
impl<'a> From<&'a Module> for ComponentRef<'a> {
    #[inline]
    fn from(source: &'a Module) -> Self {
        source.as_component_ref()
    }
}
impl<'a> From<&'a EntryPoint> for ComponentRef<'a> {
    #[inline]
    fn from(source: &'a EntryPoint) -> Self {
        source.as_component_ref()
    }
}
impl<'a> From<&'a Component> for ComponentRef<'a> {
    #[inline]
    fn from(source: &'a Component) -> Self {
        source.as_component_ref()
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
    /// Borrow this component without allocating or retaining another reference.
    #[inline]
    pub fn as_component_ref(&self) -> ComponentRef<'_> {
        ComponentRef {
            ptr: NonNull::new(self.raw.as_ptr()).expect("live component interface"),
            source: ComponentSource::Component(self),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Costs, measure};

    fn inputs() -> (Session, Module, EntryPoint) {
        let global = GlobalSession::new().unwrap();
        let session = global.create_session(&SessionOptions::default()).unwrap();
        let module = session
            .load_module_from_source(
                "costs",
                "costs.slang",
                r#"
            RWStructuredBuffer<float> output;
            [shader("compute")][numthreads(1, 1, 1)]
            void main(uint3 id : SV_DispatchThreadID) { output[id.x] = float(id.x); }
        "#,
            )
            .unwrap()
            .value;
        let entry = module.find_entry_point("main").unwrap();
        (session, module, entry)
    }

    #[test]
    fn borrowed_views_and_small_pointer_arrays_have_no_wrapper_costs() {
        let (session, module, entry) = inputs();
        let component = module.as_component();
        let (_, costs) = measure(|| {
            let views = [
                module.as_component_ref(),
                entry.as_component_ref(),
                component.as_component_ref(),
            ];
            let copied = views;
            let from = [
                ComponentRef::from(&module),
                ComponentRef::from(&entry),
                ComponentRef::from(&component),
            ];
            for (a, b) in copied.iter().zip(from) {
                assert_eq!(a.as_ptr(), b.as_ptr());
            }
            let pointers = session.component_pointers(copied.into_iter()).unwrap();
            std::hint::black_box(pointers);
        });
        assert_eq!(costs, Costs::default());
        let (result, costs) = measure(|| session.compose_refs(&[]));
        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        assert_eq!(costs, Costs::default());
        let other = GlobalSession::new()
            .unwrap()
            .create_session(&SessionOptions::default())
            .unwrap();
        let (result, costs) =
            measure(|| other.compose_refs(&[module.as_component_ref(), entry.as_component_ref()]));
        assert!(matches!(result, Err(Error::InvalidArgument(_))));
        assert_eq!(costs, Costs::default());
    }

    #[test]
    fn successful_small_composition_only_allocates_its_dependency_owner() {
        let (session, module, entry) = inputs();
        let (output, costs) = measure(|| {
            session
                .compose_refs(&[module.as_component_ref(), entry.as_component_ref()])
                .unwrap()
        });
        assert!(output.diagnostics.is_empty());
        assert_eq!(
            costs,
            Costs {
                allocations: 1,
                add_refs: 2,
                releases: 0
            }
        );
        // The same inline path is used by the existing owned-input API.
        let owned = [module.as_component(), entry.as_component()];
        let (legacy, costs) = measure(|| session.compose(&[&owned[0], &owned[1]]).unwrap());
        assert!(legacy.diagnostics.is_empty());
        assert_eq!(
            costs,
            Costs {
                allocations: 1,
                add_refs: 2,
                releases: 0
            }
        );
        drop((module, entry, owned, session, legacy));
        let code = output
            .value
            .link()
            .unwrap()
            .value
            .entry_point_code(0, 0)
            .unwrap()
            .value;
        assert_eq!(&code.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    }

    #[test]
    fn nested_composition_raw_query_retains_and_releases_session_graph() {
        let (session, module, entry) = inputs();
        let weak_session = Rc::downgrade(&session.0);
        let first = session
            .compose_refs(&[module.as_component_ref()])
            .unwrap()
            .value;
        let second = session
            .compose_refs(&[first.as_component_ref(), entry.as_component_ref()])
            .unwrap()
            .value;
        let raw = second.as_raw().clone();
        drop((first, second, entry, module, session));
        assert!(weak_session.upgrade().is_some());
        let extension = raw.query::<sys::slang_IComponentType2>().unwrap();
        drop(raw);
        // SAFETY: the retained graph has one entry and target zero; this SDK
        // call transfers owned result and diagnostic references.
        let result = unsafe {
            extension.call_owned(|ptr, out, diag| {
                sys::slang_rs_slang_IComponentType2_getEntryPointCompileResult(ptr, 0, 0, out, diag)
            })
        }
        .unwrap()
        .map(CompileResult)
        .value;
        drop(extension);
        let blob = result.item(0).unwrap();
        drop(result);
        assert!(weak_session.upgrade().is_some());
        assert_eq!(&blob.as_bytes()[..4], &0x07230203u32.to_le_bytes());
        drop(blob);
        assert!(weak_session.upgrade().is_none());
    }
}
