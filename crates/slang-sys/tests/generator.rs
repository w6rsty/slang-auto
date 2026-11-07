#[path = "../generate.rs"]
mod generate;

use std::{fs, path::PathBuf, process::Command};

/// Compile and execute a newly introduced API without adding binding declarations.
/// Exercises overloads, C++ references, record returns, function pointers, and
/// multiple inheritance (where blindly casting an interface pointer is wrong).
#[test]
fn new_cpp_api_is_discovered_and_callable() {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("generator-test")
        .join(std::process::id().to_string());
    fs::create_dir_all(&out).unwrap();
    let out = out.canonicalize().unwrap();
    // Rust 1.85 passes a verbatim current_dir through to MSVC. Its relative
    // source-file lookup then fails even though the files exist. Keep a fully
    // resolved path, but spell it as a normal Win32 drive/UNC path for tools.
    #[cfg(windows)]
    let out = {
        let path = out.to_string_lossy();
        if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
            PathBuf::from(format!(r"\\{unc}"))
        } else {
            PathBuf::from(path.strip_prefix(r"\\?\").unwrap_or(&path))
        }
    };
    let header = out.join("fixture.hpp");
    fs::write(&header, r#"
#pragma once
#include "slang.h"
namespace fixture {
struct Data { int value = 17; };
struct Left { virtual int left() { return 5; } };
struct Future : Left, ISlangUnknown {
    int refs = 1;
    static SlangUUID getTypeGuid() { return ISlangUnknown::getTypeGuid(); }
    SlangResult SLANG_MCALL queryInterface(const SlangUUID&, void** out) override { *out = static_cast<ISlangUnknown*>(this); addRef(); return 0; }
    uint32_t SLANG_MCALL addRef() override { return ++refs; }
    uint32_t SLANG_MCALL release() override { return --refs; }
    int newApi(int x) { return x + 7; }
    int newApi(const char* x) { return x[0]; }
    void references(int& number, const char*& text, bool& flag) { number = 42; text = "future"; flag = true; }
    Data makeData() { return Data{}; }
    int takeData(Data value) { return value.value; }
    int callback(int (*fn)(int)) { return fn(4); }
    static int staticApi() { return 19; }
};
inline int freeApi(int v) { return v * 2; }
}
"#).unwrap();
    let args = vec![
        "-x".into(),
        "c++".into(),
        "-std=c++17".into(),
        format!("-I{}", env!("SLANG_BINDINGS_INCLUDE")),
        format!("--target={}", env!("SLANG_BINDINGS_TARGET")),
    ];
    // Exercise both guard cleanup and thread-local teardown. An LLVM DLL may
    // load/parse successfully yet crash when its loading thread exits. Running
    // twice also detects accidentally leaking Clang's exclusive-use guard.
    for pass in 1..=2 {
        let (header, out, args) = (header.clone(), out.clone(), args.clone());
        eprintln!("Generating fixture on worker thread (pass {pass})");
        std::thread::spawn(move || {
            generate::generate(&header, &out, &out, &args, "#include \"fixture.hpp\"\n");
        })
        .join()
        .expect("generator worker panicked");
    }
    let manifest = fs::read_to_string(out.join("api-manifest.txt")).unwrap();
    for name in [
        "fixture_Future_newApi",
        "fixture_Future_newApi_1",
        "fixture_Future_references",
        "fixture_Future_callback",
        "fixture_freeApi",
    ] {
        assert!(manifest.contains(name), "Missing {name}");
    }
    let main = out.join("main.cpp");
    fs::write(
        &main,
        r#"
#include "bridge.hpp"
#include <cassert>
int square(int v) { return v*v; }
int main() {
    fixture::Future obj;
    assert(slang_rs_fixture_Future_newApi(&obj, 3) == 10);
    assert(slang_rs_fixture_Future_newApi_1(&obj, "A") == 65);
    int n=0; const char* text=nullptr; bool flag=false;
    slang_rs_fixture_Future_references(&obj,&n,&text,&flag);
    assert(n==42 && text[0]=='f' && flag);
    fixture::Data data;
    slang_rs_fixture_Future_makeData(&obj,&data);
    assert(slang_rs_fixture_Future_takeData(&obj,&data)==17);
    assert(slang_rs_fixture_Future_callback(&obj,square)==16);
    assert(slang_rs_fixture_Future_staticApi()==19);
    assert(slang_rs_fixture_freeApi(3)==6);
    ISlangUnknown* unknown = slang_rs_fixture_Future_as_unknown(&obj);
    assert(static_cast<void*>(unknown) != static_cast<void*>(&obj));
    unknown->addRef(); assert(obj.refs==2);
    unknown->release(); assert(obj.refs==1);
}
"#,
    )
    .unwrap();
    let compiler = cc::Build::new()
        .cpp(true)
        .cargo_metadata(false)
        .target(env!("SLANG_BINDINGS_TARGET"))
        .host(env!("SLANG_BINDINGS_HOST"))
        .opt_level(0)
        .debug(false)
        .out_dir(&out)
        .get_compiler();
    let exe = out.join(if cfg!(windows) {
        "fixture.exe"
    } else {
        "fixture"
    });
    let mut command = compiler.to_command();
    command.current_dir(&out);
    if compiler.is_like_msvc() {
        command
            .args(["/nologo", "/std:c++17", "/EHsc"])
            .arg(format!("/I{}", env!("SLANG_BINDINGS_INCLUDE")))
            .arg("/Fefixture.exe");
    } else {
        command
            .arg("-std=c++17")
            .arg(format!("-I{}", env!("SLANG_BINDINGS_INCLUDE")))
            .arg("-o")
            .arg(&exe);
    }
    let output = command.arg("main.cpp").arg("bridge.cpp").output().unwrap();
    assert!(
        output.status.success(),
        "C++ compilation failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(exe).output().unwrap();
    assert!(
        output.status.success(),
        "Generated C++ fixture failed ({}):\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
