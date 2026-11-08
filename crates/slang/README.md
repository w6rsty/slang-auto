# slang-auto

Rust 2024 bindings for the Slang shader compiler, with header-generated C++ API
coverage and an ergonomic ownership layer. Requires Rust **1.85+**, libclang,
a C++17 compiler, and a matching Slang SDK.

```toml
[dependencies]
slang-auto = "0.1.0"
```

Configure `SLANG_DIR` with the SDK root, or use `VULKAN_SDK`. Nonstandard layouts
can use `SLANG_INCLUDE_DIR` and `SLANG_LIB_DIR`. At runtime, put the matching SDK's
`bin` directory on `PATH` (Windows), or its library directory on
`LD_LIBRARY_PATH` (Linux). Set `LIBCLANG_PATH` if libclang is not discovered.

```rust,no_run
use slang_auto::{GlobalSession, SessionOptions};

fn main() -> slang_auto::Result<()> {
    println!("Slang build: {}", slang_auto::build_tag_string().to_string_lossy());
    let global = GlobalSession::new()?;
    let session = global.create_session(&SessionOptions::default())?;
    let module = session.load_module_from_source("example", "example.slang", r#"
        RWStructuredBuffer<float> output;
        [shader("compute")] [numthreads(8, 1, 1)]
        void main(uint3 id : SV_DispatchThreadID) { output[id.x] = 1.0; }
    "#)?;
    eprint!("{}", module.diagnostics);
    let entry = module.value.find_entry_point("main")?;
    let program = session.compose(&[&module.value.as_component(), &entry.as_component()])?;
    eprint!("{}", program.diagnostics);
    let linked = program.value.link()?;
    eprint!("{}", linked.diagnostics);
    let code = linked.value.entry_point_code(0, 0)?;
    eprint!("{}", code.diagnostics);
    println!("{} SPIR-V bytes", code.value.as_bytes().len());
    Ok(())
}
```

`ComPtr<T>` manages reference counts and retains parent objects. `query()` and
unsafe `call_owned()` support advanced interfaces without adding handwritten
bindings. Reflection views borrow their owning component. Strings validate NULs;
successful diagnostics are retained in `Output<T>`. Handles are not `Send`/`Sync`.

For explicit compiler configuration, borrow ordinary Rust strings and slices:

```rust,no_run
use slang_auto::{CompilerOption, GlobalSession, MatrixLayout, OptimizationLevel,
    SessionDesc, TargetDesc};

let global = GlobalSession::new()?;
let profile = global.find_profile("spirv_1_5")?;
let capability = global.find_capability("spvDescriptorHeapEXT")?;
let options = [
    CompilerOption::capability(capability),
    CompilerOption::optimization(OptimizationLevel::High),
    CompilerOption::define("FEATURE_X", "1"),
    CompilerOption::emit_spirv_directly(true),
];
let targets = [TargetDesc::spirv().profile(profile)];
let session = global.create_session_desc(
    &SessionDesc::new()
        .targets(&targets)
        .compiler_options(&options)
        .matrix_layout(MatrixLayout::ColumnMajor),
)?;
# Ok::<(), slang_auto::Error>(())
```

`SessionDesc` and `TargetDesc` borrow their inputs without allocation. Slang copies
configuration during session creation, so those inputs can then be dropped.
Session and target compiler options are separate; target options are passed to
their respective SDK target descriptors. `SessionDesc::new()` has **no targets**;
the existing owned `SessionOptions::default()` still selects SPIR-V.

The wrapper prepares native arrays and NUL-terminated text at the call boundary.
Small configurations use inline scratch storage (4 targets, 16 total compiler
options, 8 search paths, 8 macro descriptors, and 1024 string bytes including NULs);
larger inputs spill to the heap. Embedded NULs return `Error::InteriorNul`.
Optional `_cstr` query/option constructors skip string copying. This is not a
claim of allocation-free compilation: Slang, parent retention, and diagnostic
text conversion still have their existing costs.

`Component::entry_point_compile_result(entry, target)` and
`target_compile_result(target)` return `Output<CompileResult>` for multi-blob
outputs such as separate debug information. Use `item_count()` and `item(index)`
to retrieve owned blobs without copying their bytes, or `metadata()` for the
owned metadata interface. Results and blobs retain their dependencies. Invalid
indices are errors; extension-query failures report `Error::InterfaceQuery` with
the interface name and SDK status code. Item roles/order are target-specific,
not a universal code/debug convention. See `examples/compile.rs` for a runnable
configuration and multi-output example.

`build_tag_string()` returns the loaded compiler's build identifier as an
allocation-free `&'static CStr`, without requiring a session. Use `to_str()` for
checked UTF-8 or `to_string_lossy()` for display. This is an opaque runtime build
identifier; custom builds need not use semantic-version formatting.

The safe layer covers common operations; all generated compiler and reflection
APIs remain available through `sys`. C++ templates and Rust implementations of
C++ virtual interfaces are outside automatic coverage. Bindings follow the chosen
SDK; keep its headers and runtime libraries at the same version. This is not a
drop-in replacement for `shader-slang`.

See the [project README](https://github.com/w6rsty/slang-auto#readme) for SDK discovery,
advanced APIs, compatibility, CI, and release instructions. docs.rs uses an SDK
2026.17.1 Windows x64 API snapshot for documentation only; normal builds always
generate bindings from your installed SDK.

Licensed under MIT. Slang and other dependencies retain their own licenses.
