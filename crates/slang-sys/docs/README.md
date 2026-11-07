# Documentation-only declarations

`windows-x86_64/` contains generated declarations for docs.rs, targeting
`x86_64-pc-windows-msvc` and the SDK version in `slang-tag-version.h`.
No native SDK binaries are included. Normal builds regenerate bindings from
the configured SDK and never use this snapshot.

On a Windows x64 build host, refresh with:

```powershell
python scripts/update_docs_snapshot.py --sdk C:/path/to/slang
```

The generated bindings include Slang API declarations and documentation.
Upstream: https://github.com/shader-slang/slang
These upstream portions retain **Apache-2.0 WITH LLVM-exception**; see
`SLANG-LICENSE` and `SLANG-APACHE-2.0.txt`. The Rust generator and ownership layer
are original project code licensed under MIT.
