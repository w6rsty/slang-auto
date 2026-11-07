# slang-auto-sys

Raw Slang shader compiler and reflection bindings generated from the selected
SDK's C++ headers. Rust 2024 edition, MSRV **1.85**. Most applications should use
[`slang-auto`](https://crates.io/crates/slang-auto) instead.

At build time, Clang visits public concrete functions and methods, emits C ABI
adapters, and bindgen generates the Rust declarations. A C++17 compiler builds the
adapters, so Rust does not maintain a handwritten VTable. The SDK's interface
UUIDs and reference-counting implementations are generated as well.

Requirements: matching Slang headers and libraries, libclang, and a C++17 compiler.
Set `SLANG_DIR`, `VULKAN_SDK`, or `SLANG_INCLUDE_DIR` + `SLANG_LIB_DIR`. Set
`LIBCLANG_PATH` when libclang cannot be found. Runtime shared libraries must be
available through your platform's loader path. Native dependencies are not
downloaded by this crate's build script.

Every raw call is unsafe: callers must uphold Slang's pointer validity, ownership,
threading, and buffer contracts. C++ reference arguments become pointers and
record returns use output pointers. Default initialization uses the C++ SDK's
aggregate initializers. See `API_MANIFEST` for generated coverage and template
exclusions. This is compiler/reflection API coverage, not the GPU gfx library.

docs.rs uses a checked-in Windows x64 / Slang 2026.17.1 declaration snapshot, with
no SDK download or native linking. It is not used for ordinary builds. SDK-derived
documentation carries its upstream notices in `docs/`.

[Project documentation](https://github.com/w6rsty/slang-auto#readme).
Original project code is licensed under MIT; upstream Slang material retains its
Apache-2.0 WITH LLVM-exception license.
