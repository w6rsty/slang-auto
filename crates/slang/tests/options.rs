use slang_auto::{
    CompilerOption, DebugInfoLevel, Error, GlobalSession, MatrixLayout, OptimizationLevel,
    SessionDesc, Stage, TargetDesc,
};

const SOURCE: &str = r#"
#include "settings.slangh"
#if FROM_DEFINE != 11 || FROM_OPTION != 17 || FROM_CSTR != 23
#error Session configuration was not retained
#endif
RWStructuredBuffer<float> output;
[numthreads(GROUP_SIZE, 1, 1)]
void dispatch_main(uint3 id : SV_DispatchThreadID) { output[id.x] = float(id.x); }
"#;

#[test]
fn borrowed_text_and_target_options_survive_descriptor_drop() {
    let global = GlobalSession::new().unwrap();
    let session = {
        let path = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
        let paths = [path.as_str()];
        let defines = [("FROM_DEFINE", "11")];
        let options = [
            CompilerOption::define("FROM_OPTION", "17"),
            CompilerOption::define_cstr(c"FROM_CSTR", c"23"),
            CompilerOption::vulkan_use_entry_point_name(false),
            CompilerOption::optimization(OptimizationLevel::None),
        ];
        let named = [CompilerOption::vulkan_use_entry_point_name(true)];
        let profile = global.find_profile("spirv_1_5").unwrap();
        let targets = [
            TargetDesc::spirv_asm()
                .profile(profile)
                .compiler_options(&named),
            TargetDesc::spirv_asm().profile(profile),
        ];
        global
            .create_session_desc(
                &SessionDesc::new()
                    .targets(&targets)
                    .search_paths(&paths)
                    .defines(&defines)
                    .compiler_options(&options)
                    .matrix_layout(MatrixLayout::ColumnMajor),
            )
            .unwrap()
    };
    drop(global);
    let module = session
        .load_module_from_source("config_test", "config_test.slang", SOURCE)
        .unwrap()
        .value;
    let entry = module
        .find_and_check_entry_point("dispatch_main", Stage::Compute.into())
        .unwrap()
        .value;
    let program = session
        .compose(&[&module.as_component(), &entry.as_component()])
        .unwrap()
        .value
        .link()
        .unwrap()
        .value;
    assert_eq!(
        program
            .layout(0)
            .unwrap()
            .value
            .entry_point(0)
            .unwrap()
            .thread_group_size(),
        [4, 1, 1]
    );
    for (target, expected) in [(0, "\"dispatch_main\""), (1, "\"main\"")] {
        let code = program.entry_point_code(0, target).unwrap().value;
        let text = code.as_str().unwrap();
        let entry = text
            .lines()
            .find(|line| line.contains("OpEntryPoint"))
            .unwrap();
        assert!(entry.contains(expected), "{entry}");
    }
}

#[test]
fn all_string_locations_reject_nuls_and_unknown_names_fail() {
    let global = GlobalSession::new().unwrap();
    let targets = [TargetDesc::spirv()];
    let base = SessionDesc::new().targets(&targets);
    let paths = ["bad\0path"];
    let defines = [("name", "bad\0value")];
    let bad_name = [("bad\0name", "value")];
    let options = [CompilerOption::define("bad\0name", "1")];
    let bad_value = [CompilerOption::define("name", "bad\0value")];
    let include = [CompilerOption::include_path("bad\0path")];
    let bad_target = [TargetDesc::spirv().compiler_options(&bad_value)];
    for desc in [
        base.search_paths(&paths),
        base.defines(&defines),
        base.defines(&bad_name),
        base.compiler_options(&options),
        base.compiler_options(&bad_value),
        base.compiler_options(&include),
        base.targets(&bad_target),
    ] {
        assert!(matches!(
            global.create_session_desc(&desc),
            Err(Error::InteriorNul(_))
        ));
    }
    assert!(matches!(
        global.find_capability("bad\0capability"),
        Err(Error::InteriorNul(_))
    ));
    assert!(
        global
            .find_capability("not_a_real_slang_capability")
            .is_err()
    );
    assert!(
        global
            .find_profile_cstr(c"not_a_real_slang_profile")
            .is_err()
    );
    assert_eq!(
        global.find_profile("spirv_1_5").unwrap(),
        global.find_profile_cstr(c"spirv_1_5").unwrap()
    );
    assert_eq!(
        global.find_capability("spvDescriptorHeapEXT").unwrap(),
        global
            .find_capability_cstr(c"spvDescriptorHeapEXT")
            .unwrap()
    );
}

#[test]
fn compile_results_check_indices_and_retain_owned_outputs() {
    let global = GlobalSession::new().unwrap();
    let options = [
        CompilerOption::emit_spirv_directly(true),
        CompilerOption::debug_information(DebugInfoLevel::Standard),
        CompilerOption::emit_separate_debug(true),
    ];
    let targets = [TargetDesc::spirv()];
    let session = global
        .create_session_desc(
            &SessionDesc::new()
                .targets(&targets)
                .compiler_options(&options),
        )
        .unwrap();
    let module = session
        .load_module_from_source(
            "results",
            "results.slang",
            r#"
        RWStructuredBuffer<float> output;
        [shader("compute")][numthreads(1, 1, 1)]
        void main(uint3 id : SV_DispatchThreadID) { output[id.x] = float(id.x); }
    "#,
        )
        .unwrap()
        .value;
    let entry = module.find_entry_point("main").unwrap();
    let program = session
        .compose(&[&module.as_component(), &entry.as_component()])
        .unwrap()
        .value
        .link()
        .unwrap()
        .value;
    assert!(matches!(
        program.entry_point_compile_result(usize::MAX, 0),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        program.entry_point_compile_result(0, usize::MAX),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        program.target_compile_result(usize::MAX),
        Err(Error::InvalidArgument(_))
    ));
    let result = program.entry_point_compile_result(0, 0).unwrap().value;
    let target_result = program.target_compile_result(0).unwrap().value;
    drop((program, entry, module, session, global));
    assert_eq!(result.item_count(), 2);
    assert!(matches!(result.item(2), Err(Error::InvalidArgument(_))));
    assert!(matches!(
        result.item(usize::MAX),
        Err(Error::InvalidArgument(_))
    ));
    let metadata = result.metadata().unwrap();
    let code = result.item(0).unwrap();
    let debug = result.item(1).unwrap();
    assert_eq!(target_result.item_count(), 2);
    drop((result, target_result, metadata));
    for blob in [code, debug] {
        assert_eq!(&blob.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    }
}
