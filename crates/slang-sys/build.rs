mod generate;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn setting(key: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={key}");
    env::var(key).ok().filter(|s| !s.is_empty())
}

fn first_dir(root: &Path, suffixes: &[&str], header: bool) -> Option<PathBuf> {
    suffixes.iter().map(|s| root.join(s)).find(|p| {
        if header {
            p.join("slang.h").is_file()
        } else {
            p.is_dir()
        }
    })
}

fn main() {
    // docs.rs has no Slang SDK and forbids network access. This snapshot is only
    // for rustdoc; normal builds always generate and link against the user's SDK.
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var_os("DOCS_RS").is_some() {
        assert_eq!(
            env::var("TARGET").unwrap(),
            "x86_64-pc-windows-msvc",
            "The docs.rs snapshot targets Windows x64; use --target x86_64-pc-windows-msvc"
        );
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        for name in ["bindings.rs", "interfaces.rs", "api-manifest.txt"] {
            let source = PathBuf::from("docs/windows-x86_64").join(name);
            println!("cargo:rerun-if-changed={}", source.display());
            fs::copy(source, out.join(name)).expect("Missing docs.rs snapshot");
        }
        return;
    }
    setting("LIBCLANG_PATH");
    let explicit_root = setting("SLANG_DIR").map(PathBuf::from);
    let vulkan = setting("VULKAN_SDK").map(PathBuf::from);
    let include_override = setting("SLANG_INCLUDE_DIR").map(PathBuf::from);
    let lib_override = setting("SLANG_LIB_DIR").map(PathBuf::from);
    let lib_name = setting("SLANG_LIB_NAME").unwrap_or_else(|| "slang".into());
    // An explicit SDK is authoritative: never silently mix it with another install.
    let mut roots: Vec<PathBuf> = explicit_root.clone().or(vulkan).into_iter().collect();
    if explicit_root.is_none() {
        if let Some(profile) = setting("USERPROFILE") {
            roots.push(PathBuf::from(profile).join("scoop/apps/slang/current"));
        }
        if let Some(path) = setting("PATH") {
            roots.extend(env::split_paths(&path).filter_map(|p| {
                if p.join(if cfg!(windows) {
                    "slangc.exe"
                } else {
                    "slangc"
                })
                .is_file()
                {
                    p.parent().map(Path::to_owned)
                } else {
                    None
                }
            }));
        }
        roots.extend([PathBuf::from("/usr/local"), PathBuf::from("/usr")]);
    }
    let root = roots.iter().find(|r| {
        first_dir(
            r,
            &["include", "include/slang", "Include", "Include/slang"],
            true,
        )
        .is_some()
    });
    assert!(
        include_override.is_none() || lib_override.is_some() || explicit_root.is_some(),
        "When overriding SLANG_INCLUDE_DIR, also set SLANG_LIB_DIR or SLANG_DIR to select the matching library"
    );
    let include = include_override
        .or_else(|| {
            root.and_then(|r| {
                first_dir(
                    r,
                    &["include", "include/slang", "Include", "Include/slang"],
                    true,
                )
            })
        })
        .expect(
            "Slang SDK not found: set SLANG_DIR, VULKAN_SDK, or SLANG_INCLUDE_DIR + SLANG_LIB_DIR",
        );
    assert!(
        include.join("slang.h").is_file(),
        "SLANG_INCLUDE_DIR must contain slang.h"
    );
    let include = absolute_path(&include);
    let lib = lib_override
        .or_else(|| root.and_then(|r| first_dir(r, &["lib", "lib64", "Lib"], false)))
        .expect("Set SLANG_LIB_DIR to the directory containing the matching Slang library");
    assert!(lib.is_dir(), "SLANG_LIB_DIR must be an existing directory");
    let lib = absolute_path(&lib);
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let mut args = vec![
        "-x".into(),
        "c++".into(),
        "-std=c++17".into(),
        format!("-I{}", include.display()),
        format!("--target={target}"),
    ];
    if let Some(extra) = setting("SLANG_CLANG_ARGS") {
        // One argument per line, allowing paths with spaces without shell quoting.
        args.extend(extra.lines().filter(|a| !a.is_empty()).map(str::to_owned));
    }
    let mut headers = vec!["slang.h".to_string(), "slang-deprecated.h".to_string()];
    if let Some(extra) = setting("SLANG_EXTRA_HEADERS") {
        headers.extend(
            extra
                .split(';')
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
        );
    }
    for header in &headers {
        println!("cargo:rerun-if-changed={}", include.join(header).display());
    }
    let umbrella = out.join("input.hpp");
    let includes: String = headers
        .iter()
        .map(|h| format!("#include \"{h}\"\n"))
        .collect();
    fs::write(&umbrella, &includes).unwrap();
    generate::generate(&umbrella, &include, &out, &args, &includes);
    let bridge = out.join("bridge.hpp");
    bindgen::Builder::default()
        .header(bridge.to_str().unwrap())
        .clang_args(&args)
        .rust_target(bindgen::RustTarget::stable(85, 0).unwrap())
        .rust_edition(bindgen::RustEdition::Edition2024)
        .allowlist_function("slang_rs_.*|sp.*|slang_.*")
        .allowlist_type("slang::.*|Slang.*|ISlang.*")
        .allowlist_var("SLANG_.*|slang::.*|k.*")
        .allowlist_file(".*[/\\\\]slang[^/\\\\]*\\.h")
        .opaque_type("std::.*")
        .with_codegen_config(
            bindgen::CodegenConfig::FUNCTIONS
                | bindgen::CodegenConfig::TYPES
                | bindgen::CodegenConfig::VARS,
        )
        .layout_tests(true)
        .derive_default(false)
        // bridge.hpp is generated in OUT_DIR, not a source dependency. Tracking
        // it makes this script invalidate itself on every build. Included SDK
        // and system headers, and bindgen environment variables, remain tracked.
        .parse_callbacks(Box::new(
            bindgen::CargoCallbacks::new().rerun_on_header_files(false),
        ))
        .generate()
        .expect("bindgen failed")
        .write_to_file(out.join("bindings.rs"))
        .unwrap();
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .include(&include)
        .include(&out)
        .file(out.join("bridge.cpp"))
        .warnings(false);
    // Apply custom include/define arguments to the bridge as well as the AST parser.
    for arg in args.iter().skip(5) {
        build.flag(arg);
    }
    build.compile("slang_rs_bridge");
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=dylib={lib_name}");
    println!("cargo:include={}", include.display());
    println!(
        "cargo:rustc-env=SLANG_BINDINGS_INCLUDE={}",
        include.display()
    );
    println!("cargo:rustc-env=SLANG_BINDINGS_TARGET={target}");
    println!(
        "cargo:rustc-env=SLANG_BINDINGS_HOST={}",
        env::var("HOST").unwrap()
    );
    println!(
        "cargo:warning=Slang bindings generated from {}",
        include.display()
    );
}

fn absolute_path(path: &Path) -> PathBuf {
    let path = path.canonicalize().expect("Cannot resolve SDK path");
    // MSVC's command-line parser does not consistently accept verbatim paths.
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{unc}"));
        }
        if let Some(drive) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(drive);
        }
    }
    path
}
