# slang-auto

[![CI](https://github.com/w6rsty/slang-auto/actions/workflows/ci.yml/badge.svg)](https://github.com/w6rsty/slang-auto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

| crate | Purpose |
| --- | --- |
| `slang-auto-sys` | Clang AST -> C++ C ABI bridge -> bindgen Rust declarations; the complete low-level API |
| `slang-auto` | Reference counting, strings, diagnostics, compilation workflow, and reflection views with lifetimes |

## Quick Start

Building requires Rust 1.85 or later, libclang, a C++17 compiler, and the Slang SDK. On Windows, use the MSVC Build Tools and Windows SDK; LLVM provides `libclang.dll`. The SDK headers, libraries, and runtime DLLs must all come from the same version and target architecture.

Standalone local SDK:

```powershell
$env:SLANG_DIR = 'C:/Users/worse/scoop/apps/slang/2026.17.1'
$env:LIBCLANG_PATH = 'C:/Users/worse/scoop/apps/llvm/current/bin'
$env:PATH = "$env:SLANG_DIR/bin;$env:PATH"
cargo run -p slang-auto --example compile
cargo test --workspace
```

To use the Vulkan SDK instead:

```powershell
$env:SLANG_DIR = $null
$env:VULKAN_SDK = 'C:/VulkanSDK/1.4.357.0'
$env:LIBCLANG_PATH = 'C:/Users/worse/scoop/apps/llvm/current/bin'
$env:PATH = "$env:VULKAN_SDK/Bin;$env:PATH"
cargo run -p slang-auto --example compile
```

Linux and macOS use the same configuration variables. At runtime, set `LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` as appropriate for the system, or install the libraries where the dynamic loader can find them. Windows x64 MSVC is the locally tested platform. Linux x64 has been added to GitHub Actions, but its first remote run must be confirmed after pushing. macOS is not yet included in CI.


```rust
use slang_auto::{GlobalSession, SessionOptions};

fn main() -> slang_auto::Result<()> {
    let global = GlobalSession::new()?;
    let session = global.create_session(&SessionOptions::default())?;
    let loaded = session.load_module_from_source("compute", "compute.slang", r#"
        RWStructuredBuffer<float> output;
        [shader("compute")]
        [numthreads(8, 1, 1)]
        void main(uint3 tid : SV_DispatchThreadID) { output[tid.x] = 2.0; }
    "#)?;
    eprint!("{}", loaded.diagnostics);
    let entry = loaded.value.find_entry_point("main")?;
    let composed = session.compose(&[
        &loaded.value.as_component(), &entry.as_component(),
    ])?;
    eprint!("{}", composed.diagnostics);
    let linked = composed.value.link()?;
    eprint!("{}", linked.diagnostics);
    let layout = linked.value.layout(0)?;
    eprint!("{}", layout.diagnostics);
    for parameter in layout.value.parameters() {
        println!("{:?}: set {}, binding {}",
            parameter.name(), parameter.binding_space(), parameter.binding_index());
    }
    let code = linked.value.entry_point_code(0, 0)?;
    eprint!("{}", code.diagnostics);
    println!("{} SPIR-V bytes", code.value.as_bytes().len());
    Ok(())
}
```

The complete runnable example is in `crates/slang/examples/compile.rs`.

### Slang Build Identifier

Query the loaded compiler library without creating a session:

```rust
let tag = slang_auto::build_tag_string(); // &'static CStr, no allocation
println!("Slang build: {}", tag.to_string_lossy());
```

This wraps `spGetBuildTagString()`, which returns the same identifier as
`IGlobalSession::getBuildTagString()`. The bytes belong to the linked library and
remain valid independently of session lifetimes. Use `tag.to_str()` for checked
UTF-8 or `tag.to_bytes()` for exact bytes. The identifier describes the runtime
library, not the Rust crate or build-time SDK headers. Custom Slang builds can
return arbitrary text or a timestamp; do not assume semantic-version formatting.

## SDK Configuration

| Environment variable | Purpose |
| --- | --- |
| `SLANG_DIR` | SDK root directory; when explicitly set, no other SDK is used as a fallback |
| `VULKAN_SDK` | Searched first when `SLANG_DIR` is not set; supports `Include/slang` |
| `SLANG_INCLUDE_DIR` | Explicitly specifies the directory containing `slang.h`; also set `SLANG_LIB_DIR` or `SLANG_DIR` |
| `SLANG_LIB_DIR` | Library directory for linking; overrides the directory inferred from the SDK root |
| `SLANG_LIB_NAME` | Dynamic library name for linking, defaulting to `slang`, without prefix or suffix |
| `LIBCLANG_PATH` | Directory containing the libclang shared library |
| `SLANG_CLANG_ARGS` | Additional compiler arguments, one per line; for example, `-DMY_DEFINE=1`. Applied to both the parser and the C++ bridge |
| `SLANG_EXTRA_HEADERS` | Additional header filenames in the SDK include directory, separated by semicolons; for extensions, they must satisfy the generator's supported C++ type constraints |

If `SLANG_DIR` is not explicitly set, the following locations are tried in order: `VULKAN_SDK`, the user's Scoop `slang/current`, the installation root of `slangc` found on `PATH`, `/usr/local`, and `/usr`. If the directory layout differs, override it with the two explicit paths. The C++ compiler can be configured with the `cc` crate's `CXX` / `CXXFLAGS`; Clang parsing and actual C++ compilation should use the same macros, headers, and target ABI.

Cargo tracks all environment variables and headers for rebuilds. After switching SDKs, simply rebuild; there is no need to regenerate or edit source files. Bindings target the SDK version used at build time, so do not mix them with a runtime from another version.

## API Coverage and Generation Rules

The default inputs are `slang.h` and `slang-deprecated.h`. All public, non-template compilation and reflection functions and methods in these headers are covered, including the C API, inline C++ reflection helpers, COM interfaces, experimental extensions, and the legacy compilation request API. Types, enums, and constants representable by bindgen are also exported. GPU gfx and shader host/CUDA preludes are outside this API scope and are not included by default.

The Slang 2026.17.1 generation manifest contains **761 function/method bridges, 55 interface conversions, and 25 default initializers**. Two additional manifest entries are the C++ templates `DeclReflection::FilteredList<K>` and `getChildrenOfKind<K>`. They are not exported as uninstantiated templates; the same filtering can be implemented with the bound `getChildrenCount/getChild/getKind` methods. This is a documented template boundary, not a missing compiler runtime interface.

- New ordinary functions, virtual methods, and reflection methods are discovered automatically at build time; there is no per-method allowlist.
- Interface calls are executed by the C++ compiler; inheritance conversions use `static_cast` and support nonzero base-class offsets.
- `T&` is converted to a pointer parameter and dereferenced in C++; function pointers retain their corresponding signatures.
- Struct-by-value parameters are converted to input pointers, and struct returns to output pointers, avoiding differences in the C++ record return ABI.
- `Default` uses C++ `{}` initialization, including for `structureSize` and SDK defaults, rather than Rust zero-initialization.
- Suffixes such as `_1` and `_2` are appended to overloaded public methods in declaration order; the original interface names and signatures are recorded in the manifest. Changes in overload order may change the underlying Rust names.
- Bridge functions are `noexcept`: Slang errors are returned through `SlangResult` and diagnostics; unexpected C++ exceptions do not cross the Rust FFI boundary and instead terminate the process.
- Variadic functions, non-trivial C++ value types that cannot be bridged by these rules, and similar cases cause generation or compilation to fail. The generator cannot be guaranteed to support arbitrary future C++ syntax without extensions.

Each build generates the following files in `target/.../build/slang-auto-sys-*/out/`:

```text
bridge.hpp        C ABI declarations
bridge.cpp        C++ method bridge implementations
bindings.rs       bindgen type and function declarations
interfaces.rs     Automatically generated Interface and Default implementations
api-manifest.txt  Auditable API manifest, including template boundaries
```

The manifest can also be read through `slang_auto::sys::API_MANIFEST`. Use `cargo doc --workspace --no-deps` to view the generated API. All low-level functions are `unsafe`.

## Rust Boundary and Lifetimes

- `&str` inputs are checked for embedded NULs through `CString`; module source loading also provides a `&CStr` entry point to avoid duplicate allocations.
- Names are returned as `Option<&CStr>` without assuming that SDK bytes are UTF-8. Blob provides byte slices, strict UTF-8 conversion, and lossy text conversion.
- `Output<T>` preserves warnings on success; `Error::Slang` preserves diagnostics on failure. A negative `SlangResult` is treated as a failure.
- `Clone/Drop` correspond to `addRef/release`. Borrowed pointers returned when loading modules are retained; outputs with owned references are taken over directly.
- Sessions, components, and extension interfaces retain the parent objects they need; cloning an `as_raw()` handle also retains these dependencies.
- A reflection view's `'a` is tied to its component, preventing access to the reflection tree after the program is released. The safe layer checks index bounds and rejects components from different sessions during composition.
- Objects remain `!Send + !Sync`; multithreaded compilation should use a separate session on each thread. Callers remain responsible for thread and aliasing rules when invoking raw interfaces.
- Implementing custom C++ virtual interfaces in Rust is outside the scope of this project's automatic generation. Raw callbacks and filesystem interfaces can be integrated by writing a custom C++ implementation.


## License

Original code in this project is licensed under the [MIT License](LICENSE), with copyright held by w6rsty and contributors. Upstream notices and documentation included in the Slang SDK and documentation snapshot retain their Apache-2.0 WITH LLVM-exception license; related notices are in `crates/slang-sys/docs/`. Third-party dependencies retain their respective licenses and are not relicensed under MIT by this project.

Design references: [Slang upstream headers](https://github.com/shader-slang/slang/blob/master/include/slang.h), [bindgen C++ support limitations](https://rust-lang.github.io/rust-bindgen/cpp.html), and [reference Rust bindings](https://github.com/FloatyMonkey/slang-rs).
