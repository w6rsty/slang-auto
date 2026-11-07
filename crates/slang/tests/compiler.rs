use slang_auto::{Component, Error, GlobalSession, SessionOptions, sys};

const SHADER: &str = r#"
    struct Settings { float scale; float3 bias; };
    ConstantBuffer<Settings> settings;
    RWStructuredBuffer<float> output;
    [shader("compute")]
    [numthreads(8, 1, 1)]
    void main(uint3 tid : SV_DispatchThreadID) { output[tid.x] = settings.scale + settings.bias.x; }
"#;

fn linked_program() -> Component {
    let global = GlobalSession::new().unwrap();
    let session = global.create_session(&SessionOptions::default()).unwrap();
    let module = session
        .load_module_from_source("test", "test.slang", SHADER)
        .unwrap()
        .value;
    let entry = module.find_entry_point("main").unwrap();
    session
        .compose(&[&module.as_component(), &entry.as_component()])
        .unwrap()
        .value
        .link()
        .unwrap()
        .value
    // All local parents drop here. The returned program must retain them.
}

#[test]
fn compile_reflect_and_retain_parents() {
    let program = linked_program();
    let layout = program.layout(0).unwrap().value;
    assert_eq!(layout.entry_point_count(), 1);
    let entry = layout.entry_point(0).unwrap();
    assert_eq!(entry.name().unwrap().to_bytes(), b"main");
    assert_eq!(entry.thread_group_size(), [8, 1, 1]);
    assert_eq!(entry.stage(), sys::SlangStage_SLANG_STAGE_COMPUTE);
    assert!(layout.entry_point(1).is_none());
    assert!(layout.parameter(usize::MAX).is_none());
    let settings = layout.find_type("Settings").unwrap().unwrap();
    let fields: Vec<_> = settings
        .fields()
        .map(|v| v.name().unwrap().to_bytes().to_vec())
        .collect();
    assert_eq!(fields, [b"scale".to_vec(), b"bias".to_vec()]);
    let params: Vec<_> = layout.parameters().collect();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name().unwrap().to_bytes(), b"settings");
    let code = program.entry_point_code(0, 0).unwrap().value;
    assert_eq!(&code.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    assert_eq!(code.as_bytes().len() % 4, 0);
    let clone = code.clone();
    drop(code);
    drop(program);
    assert!(clone.as_bytes().len() > 20);
}

#[test]
fn extension_query_and_cpp_reference_parameter() {
    let program = linked_program();
    let extension = program
        .as_raw()
        .query::<sys::slang_IComponentType2>()
        .unwrap();
    let result = unsafe {
        extension.call_owned(|ptr, out, diag| {
            sys::slang_rs_slang_IComponentType2_getTargetCompileResult(ptr, 0, out, diag)
        })
    }
    .unwrap()
    .value;
    assert!(unsafe { sys::slang_rs_slang_ICompileResult_getItemCount(result.as_ptr()) } > 0);
    let metadata = program.target_metadata(0).unwrap().value;
    let unknown = metadata.query::<sys::ISlangUnknown>().unwrap();
    let metadata = unknown.query::<sys::slang_IMetadata>().unwrap();
    // The program and original queried owner can drop before the extension handle.
    drop(unknown);
    drop(program);
    let mut used = false;
    let result = unsafe {
        sys::slang_rs_slang_IMetadata_isParameterLocationUsed(
            metadata.as_ptr(),
            sys::SlangParameterCategory_SLANG_PARAMETER_CATEGORY_DESCRIPTOR_TABLE_SLOT,
            0,
            0,
            &mut used,
        )
    };
    assert!(result >= 0, "metadata result: {result}");
    assert!(used);
}

#[test]
fn strings_errors_warnings_and_bounds() {
    let global = GlobalSession::new().unwrap();
    assert!(matches!(
        global.find_profile("spirv\0bad"),
        Err(Error::InteriorNul(_))
    ));
    assert!(global.find_profile("not-a-profile").is_err());
    let session = global.create_session(&SessionOptions::default()).unwrap();
    assert!(matches!(
        session.load_module_from_source("bad\0", "bad.slang", SHADER),
        Err(Error::InteriorNul(_))
    ));
    let err = session
        .load_module_from_source("broken", "broken.slang", "this is invalid syntax !!!")
        .unwrap_err();
    assert!(matches!(err,Error::Slang { ref diagnostics,.. } if diagnostics.contains("error")));
    let warning = session
        .load_module_from_source(
            "warnings",
            "warnings.slang",
            &format!("#warning binding-warning\n{SHADER}"),
        )
        .unwrap();
    assert!(warning.diagnostics.contains("binding-warning"));
    assert!(warning.value.find_entry_point("absent").is_err());
    let program = linked_program();
    assert!(matches!(
        program.layout(999),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        program.entry_point_code(999, 0),
        Err(Error::InvalidArgument(_))
    ));
}

#[test]
fn rejects_cross_session_composition() {
    let global = GlobalSession::new().unwrap();
    let first = global.create_session(&SessionOptions::default()).unwrap();
    let second = global.create_session(&SessionOptions::default()).unwrap();
    let m = first
        .load_module_from_source("test", "test.slang", SHADER)
        .unwrap()
        .value;
    assert!(matches!(
        second.compose(&[&m.as_component()]),
        Err(Error::InvalidArgument(_))
    ));
    assert!(first.compose(&[]).is_err());
}

#[test]
fn descriptors_use_cpp_defaults_and_advanced_api_is_present() {
    let session = sys::slang_SessionDesc::default();
    let target = sys::slang_TargetDesc::default();
    assert_eq!(session.structureSize, std::mem::size_of_val(&session));
    assert_eq!(target.structureSize, std::mem::size_of_val(&target));
    assert_eq!(target.flags, sys::kDefaultTargetFlags);
    assert!(session.targets.is_null());
    for name in [
        "slang_ISession_loadModuleInfoFromIRBlob",
        "slang_IComponentType2_getTargetCompileResult",
        "slang_IMetadata_isParameterLocationUsed",
        "slang_TypeLayoutReflection_getBindingRangeCount",
    ] {
        assert!(
            sys::API_MANIFEST.contains(name),
            "Missing generated API: {name}"
        );
    }
}
