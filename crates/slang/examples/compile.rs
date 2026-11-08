use slang_auto::{GlobalSession, SessionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Slang build: {}",
        slang_auto::build_tag_string().to_string_lossy()
    );
    let global = GlobalSession::new()?;
    let session = global.create_session(&SessionOptions::default())?;
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
    let program = session.compose(&[&module.value.as_component(), &entry.as_component()])?;
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
    let code = linked.value.entry_point_code(0, 0)?;
    print_diagnostics(&code.diagnostics);
    println!("SPIR-V: {} bytes", code.value.as_bytes().len());
    assert_eq!(&code.value.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    Ok(())
}
fn print_diagnostics(text: &str) {
    if !text.is_empty() {
        eprintln!("{text}");
    }
}
