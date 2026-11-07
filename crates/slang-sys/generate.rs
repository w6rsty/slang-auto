//! AST-driven C ABI adapter. Never encode a C++ vtable layout in Rust.
use clang::{Accessibility, Clang, Entity, EntityKind as K, Index, Type, TypeKind as T};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn qualified(e: Entity<'_>) -> String {
    let name = e.get_name().unwrap_or_default();
    match e.get_semantic_parent() {
        Some(p)
            if matches!(
                p.get_kind(),
                K::Namespace | K::StructDecl | K::ClassDecl | K::UnionDecl
            ) =>
        {
            let parent = qualified(p);
            if parent.is_empty() {
                name
            } else {
                format!("{parent}::{name}")
            }
        }
        _ => name,
    }
}

fn ident(s: &str) -> String {
    s.replace("::", "_")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn spelling(t: Type<'_>) -> String {
    // Preserve pointer-sized standard typedefs for idiomatic bindgen usize/isize.
    match t.get_display_name().as_str() {
        "size_t" => "size_t".into(),
        "ptrdiff_t" => "ptrdiff_t".into(),
        _ => t.get_canonical_type().get_display_name(),
    }
}
fn reference(t: Type<'_>) -> bool {
    matches!(
        t.get_canonical_type().get_kind(),
        T::LValueReference | T::RValueReference
    )
}
fn record(t: Type<'_>) -> bool {
    t.get_canonical_type().get_kind() == T::Record
}

fn derives_unknown(e: Entity<'_>) -> bool {
    qualified(e) == "ISlangUnknown"
        || e.get_children()
            .into_iter()
            .filter(|c| c.get_kind() == K::BaseSpecifier)
            .any(|b| {
                b.get_type()
                    .and_then(|t| t.get_declaration())
                    .is_some_and(derives_unknown)
            })
}

#[derive(Default)]
struct Generator {
    header: String,
    source: String,
    interfaces: String,
    manifest: Vec<String>,
    seen: BTreeSet<String>,
    overloads: BTreeMap<String, usize>,
}

impl Generator {
    fn emit(&mut self, name: &str, result: &str, params: &[String], body: &str) {
        let decl = format!("{result} {name}({}) noexcept", params.join(", "));
        self.header.push_str(&format!("extern \"C\" {decl};\n"));
        self.source
            .push_str(&format!("extern \"C\" {decl} {{ {body} }}\n"));
    }

    fn function(&mut self, e: Entity<'_>, owner: Option<Entity<'_>>) {
        if matches!(
            e.get_accessibility(),
            Some(Accessibility::Private | Accessibility::Protected)
        ) {
            return;
        }
        let q = qualified(e);
        let signature = format!("{q}: {}", e.get_type().unwrap().get_display_name());
        if !self.seen.insert(signature.clone()) {
            return;
        }
        assert!(!e.is_variadic(), "Unsupported variadic API: {signature}");
        let base = format!("slang_rs_{}", ident(&q));
        let ordinal = self.overloads.entry(base.clone()).or_default();
        let name = if *ordinal == 0 {
            base
        } else {
            format!("{base}_{}", *ordinal)
        };
        *ordinal += 1;
        let mut params = Vec::new();
        let receiver = if let Some(owner) = owner {
            if e.is_static_method() {
                format!("{}::", qualified(owner))
            } else {
                let cv = if e.is_const_method() { "const " } else { "" };
                params.push(format!("{cv}{}* self", qualified(owner)));
                "self->".into()
            }
        } else {
            q.rsplit_once("::")
                .map(|(ns, _)| format!("{ns}::"))
                .unwrap_or_default()
        };
        let mut args = Vec::new();
        for (i, a) in e.get_arguments().unwrap_or_default().iter().enumerate() {
            let t = a.get_type().unwrap();
            let alias = format!("{name}_arg{i}");
            let (ty, expr) = if reference(t) {
                let ty = spelling(t.get_pointee_type().unwrap());
                if t.get_canonical_type().get_kind() == T::RValueReference {
                    (format!("{ty}*"), format!("static_cast<{ty}&&>(*a{i})"))
                } else {
                    (format!("{ty}*"), format!("*a{i}"))
                }
            } else if record(t) {
                (format!("const {}*", spelling(t)), format!("*a{i}"))
            } else {
                (spelling(t), format!("a{i}"))
            };
            self.header.push_str(&format!("using {alias} = {ty};\n"));
            params.push(format!("{alias} a{i}"));
            args.push(expr);
        }
        let ret = e.get_result_type().unwrap();
        let call = format!("{receiver}{}({})", e.get_name().unwrap(), args.join(", "));
        let (result, body) = if record(ret) {
            params.push(format!("{}* out_result", spelling(ret)));
            (
                "void".into(),
                format!(
                    "static_assert(std::is_trivially_copyable<{}>::value, \"nontrivial record return\"); ::new (static_cast<void*>(out_result)) {}({call});",
                    spelling(ret),
                    spelling(ret)
                ),
            )
        } else if reference(ret) {
            (
                format!("{}*", spelling(ret.get_pointee_type().unwrap())),
                format!("return &({call});"),
            )
        } else if ret.get_kind() == T::Void {
            ("void".into(), format!("{call};"))
        } else {
            (spelling(ret), format!("return {call};"))
        };
        let result_alias = format!("{name}_result");
        self.header
            .push_str(&format!("using {result_alias} = {result};\n"));
        self.emit(&name, &result_alias, &params, &body);
        self.manifest.push(format!("FUNCTION\t{name}\t{signature}"));
    }

    fn walk(&mut self, e: Entity<'_>, include: &Path) {
        for c in e.get_children() {
            if c.is_in_system_header() {
                continue;
            }
            let local = c
                .get_location()
                .and_then(|l| l.get_file_location().file)
                .is_some_and(|f| f.get_path().starts_with(include));
            match c.get_kind() {
                K::Namespace | K::LinkageSpec | K::UnexposedDecl => self.walk(c, include),
                K::StructDecl | K::ClassDecl | K::UnionDecl
                    if local
                        && c.is_definition()
                        && !c.is_anonymous()
                        && c.get_name().is_some() =>
                {
                    let q = qualified(c);
                    let id = ident(&q);
                    if !self.seen.insert(format!("record {q}")) {
                        continue;
                    }
                    if derives_unknown(c) {
                        let name = format!("slang_rs_{id}_as_unknown");
                        self.emit(
                            &name,
                            "ISlangUnknown*",
                            &[format!("{q}* self")],
                            "return static_cast<ISlangUnknown*>(self);",
                        );
                        self.manifest
                            .push(format!("CAST\t{name}\t{q} -> ISlangUnknown"));
                        self.interfaces.push_str(&format!(r#"
unsafe impl Interface for {id} {{
    unsafe fn add_ref(ptr: *mut Self) {{ unsafe {{ slang_rs_ISlangUnknown_addRef({name}(ptr)); }} }}
    unsafe fn release(ptr: *mut Self) {{ unsafe {{ slang_rs_ISlangUnknown_release({name}(ptr)); }} }}
    unsafe fn query_interface(ptr: *mut Self, guid: &SlangUUID, out: *mut *mut ::std::ffi::c_void) -> SlangResult {{
        unsafe {{ slang_rs_ISlangUnknown_queryInterface({name}(ptr), guid, out) }}
    }}
    fn guid() -> SlangUUID {{
        unsafe {{ let mut value = ::std::mem::MaybeUninit::uninit(); slang_rs_{id}_getTypeGuid(value.as_mut_ptr()); value.assume_init() }}
    }}
}}
"#));
                    }
                    // Explicit upcasts also work with multiple inheritance / nonzero base offsets.
                    for base in c
                        .get_children()
                        .into_iter()
                        .filter(|b| b.get_kind() == K::BaseSpecifier)
                    {
                        if matches!(
                            base.get_accessibility(),
                            Some(Accessibility::Private | Accessibility::Protected)
                        ) {
                            continue;
                        }
                        let b = spelling(base.get_type().unwrap());
                        let name = format!("slang_rs_{id}_as_{}", ident(&b));
                        self.emit(
                            &name,
                            &format!("{b}*"),
                            &[format!("{q}* self")],
                            &format!("return static_cast<{b}*>(self);"),
                        );
                        self.manifest.push(format!("CAST\t{name}\t{q} -> {b}"));
                    }
                    let has_fields = c
                        .get_children()
                        .iter()
                        .any(|f| f.get_kind() == K::FieldDecl);
                    // Value descriptors need their C++ default member initializers (not zero-fill).
                    if has_fields && !derives_unknown(c) {
                        let name = format!("slang_rs_{id}_default");
                        self.emit(&name, "void", &[format!("{q}* out_result")], &format!("static_assert(std::is_trivially_copyable<{q}>::value && std::is_aggregate<{q}>::value, \"descriptor requires explicit initialization review\"); ::new (static_cast<void*>(out_result)) {q}{{}};"));
                        self.manifest.push(format!("DEFAULT\t{name}\t{q}"));
                        self.interfaces.push_str(&format!("impl Default for {id} {{ fn default() -> Self {{ unsafe {{ let mut value = ::std::mem::MaybeUninit::uninit(); {name}(value.as_mut_ptr()); value.assume_init() }} }} }}\n"));
                    }
                    self.walk(c, include);
                }
                K::Method | K::ConversionFunction if local => {
                    self.function(c, c.get_semantic_parent())
                }
                K::FunctionDecl if local => self.function(c, None),
                K::ClassTemplate | K::FunctionTemplate if local => {
                    self.manifest.push(format!(
                        "TEMPLATE\t{}\tC++ template; use concrete reflection child/index APIs",
                        qualified(c)
                    ));
                }
                _ => {}
            }
        }
    }
}

pub fn generate(input: &Path, include: &Path, out: &Path, args: &[String], includes: &str) {
    let clang =
        Clang::new().expect("Install libclang and set LIBCLANG_PATH to its library directory");
    let index = Index::new(&clang, false, false);
    let tu = index
        .parser(input)
        .arguments(args)
        .parse()
        .expect("Cannot parse Slang headers");
    for d in tu.get_diagnostics() {
        assert!(
            d.get_severity() < clang::diagnostic::Severity::Error,
            "Slang header error: {d}"
        );
    }
    let mut g = Generator {
        header: format!("#pragma once\n{includes}#include <type_traits>\n#include <new>\n"),
        source: "#include \"bridge.hpp\"\n".into(),
        ..Default::default()
    };
    g.walk(tu.get_entity(), include);
    assert!(
        !g.interfaces.is_empty(),
        "No Slang interfaces found; check the include path"
    );
    fs::write(out.join("bridge.hpp"), g.header).unwrap();
    fs::write(out.join("bridge.cpp"), g.source).unwrap();
    fs::write(out.join("interfaces.rs"), g.interfaces).unwrap();
    fs::write(out.join("api-manifest.txt"), g.manifest.join("\n")).unwrap();
    println!(
        "cargo:warning=Generated {} Slang API entries",
        g.manifest.len()
    );
}
