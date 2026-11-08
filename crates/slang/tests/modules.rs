use slang_auto::{Blob, FileSystem, GlobalSession, SessionDesc, TargetDesc};
use std::{cell::Cell, rc::Rc, sync::Arc};

#[test]
fn owned_storage_and_filesystem_survive_parent_drop_and_ir_reloads() {
    let owned = b"allocation retained".to_vec();
    let ptr = owned.as_ptr();
    assert_eq!(Blob::from_vec(owned).as_bytes().as_ptr(), ptr);
    let bytes: Arc<[u8]> = Arc::from(b"public float shared_value() { return 4.0; }".as_slice());
    let blob = Blob::from_bytes(bytes.clone());
    assert_eq!(blob.as_bytes().as_ptr(), bytes.as_ptr());
    let lifetime = Rc::new(());
    let weak = Rc::downgrade(&lifetime);
    let reads = Rc::new(Cell::new(0));
    let counter = reads.clone();
    let fs = FileSystem::new(move |path| {
        let _owner = &lifetime;
        if path.to_bytes().ends_with(b"shared.slang") {
            counter.set(counter.get() + 1);
            Ok(Some(blob.clone()))
        } else {
            Ok(None)
        }
    });
    let global = GlobalSession::new().unwrap();
    let targets = [TargetDesc::spirv()];
    let session = global
        .create_session_desc(&SessionDesc::new().targets(&targets).filesystem(&fs))
        .unwrap();
    drop((global, fs));
    let root = session
        .load_module_from_bytes(
            "root",
            "root.slang",
            b"import shared; public float value() { return shared_value(); }",
        )
        .unwrap()
        .value;
    let modules = session.loaded_modules();
    assert_eq!(modules.len(), 2);
    let shared = modules
        .iter()
        .find(|m| m.name().unwrap().to_bytes() == b"shared")
        .unwrap();
    assert!(
        shared
            .dependency_files()
            .iter()
            .any(|p| p.to_bytes().ends_with(b"shared.slang"))
    );
    let ir = shared.serialize().unwrap();
    assert!(reads.get() > 0);
    drop((root, modules, session));
    assert!(
        weak.upgrade().is_none(),
        "serialization must not keep the old session/filesystem alive"
    );
    let global = GlobalSession::new().unwrap();
    let session = global
        .create_session_desc(&SessionDesc::new().targets(&targets))
        .unwrap();
    let restored = session
        .load_module_from_ir("shared", "shared.slang", ir.as_bytes())
        .unwrap()
        .value;
    assert_eq!(restored.name().unwrap().to_bytes(), b"shared");
}
