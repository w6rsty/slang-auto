use slang_auto::{
    CompilerOption, DebugInfoLevel, GlobalSession, OptimizationLevel, SessionDesc, TargetDesc,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Slang build: {}",
        slang_auto::build_tag_string().to_string_lossy()
    );
    let global = GlobalSession::new()?;
    let profile = global.find_profile("spirv_1_5")?;
    let options = [
        CompilerOption::optimization(OptimizationLevel::Default),
        CompilerOption::debug_information(DebugInfoLevel::Standard),
        CompilerOption::emit_spirv_directly(true),
        CompilerOption::emit_separate_debug(true),
    ];
    let targets = [TargetDesc::spirv().profile(profile)];
    let session = global.create_session_desc(
        &SessionDesc::new()
            .targets(&targets)
            .compiler_options(&options),
    )?;
    let module = session.load_module_from_source(
        "example",
        "example.slang",
        r#"
        RWStructuredBuffer<float> output;
        [shader("compute")]
        [numthreads(8, 1, 1)]
        void main(uint3 tid : SV_DispatchThreadID) { output[tid.x] = float(tid.x) * 2.0; }
    "#,
    )?;
    print_diagnostics(&module.diagnostics);
    let entry = module.value.find_entry_point("main")?;
    let program =
        session.compose_refs(&[module.value.as_component_ref(), entry.as_component_ref()])?;
    print_diagnostics(&program.diagnostics);
    let linked = program.value.link()?;
    print_diagnostics(&linked.diagnostics);
    let layout = linked.value.layout(0)?;
    print_diagnostics(&layout.diagnostics);
    for entry in layout.value.entry_points() {
        println!(
            "entry: {:?}, threads: {:?}",
            entry.name(),
            entry.thread_group_size()
        );
    }
    for parameter in layout.value.parameters() {
        println!(
            "parameter: {:?}, set: {}, binding: {}",
            parameter.name(),
            parameter.binding_space(),
            parameter.binding_index()
        );
    }
    let result = linked.value.entry_point_compile_result(0, 0)?;
    print_diagnostics(&result.diagnostics);
    for index in 0..result.value.item_count() {
        let blob = result.value.item(index)?;
        println!("SPIR-V output {index}: {} bytes", blob.as_bytes().len());
        assert_eq!(&blob.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    }
    Ok(())
}
fn print_diagnostics(text: &str) {
    if !text.is_empty() {
        eprintln!("{text}");
    }
}
