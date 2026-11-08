use slang_auto::{ComponentRef, Error, GlobalSession, SessionOptions};

#[test]
fn spilled_composition_preserves_order_and_outlives_every_input() {
    let global = GlobalSession::new().unwrap();
    let session = global.create_session(&SessionOptions::default()).unwrap();
    let mut source = String::from("RWStructuredBuffer<uint> output;\n");
    for index in 0..5 {
        source.push_str(&format!(
            "[shader(\"compute\")][numthreads({},1,1)] void entry_{index}(uint3 id:SV_DispatchThreadID) {{ output[id.x] = {index}; }}\n",
            index + 1,
        ));
    }
    let module = session
        .load_module_from_source("many_entries", "many_entries.slang", &source)
        .unwrap()
        .value;
    let entries: Vec<_> = (0..5)
        .rev()
        .map(|index| module.find_entry_point(&format!("entry_{index}")).unwrap())
        .collect();
    let program = {
        let mut views = vec![module.as_component_ref()];
        views.extend(entries.iter().map(ComponentRef::from));
        assert_eq!(views.len(), 6); // Spill both the pointer and ownership buffers.
        session.compose_refs(&views).unwrap().value
    };
    drop((entries, module, session, global, source));
    let linked = program.link().unwrap().value;
    drop(program);
    let layout = linked.layout(0).unwrap().value;
    assert_eq!(layout.entry_point_count(), 5);
    for index in 0..5 {
        let expected = 4 - index;
        let entry = layout.entry_point(index).unwrap();
        assert_eq!(
            entry.name().unwrap().to_bytes(),
            format!("entry_{expected}").as_bytes()
        );
        assert_eq!(entry.thread_group_size(), [expected as u64 + 1, 1, 1]);
        let code = linked.entry_point_code(index, 0).unwrap().value;
        assert_eq!(&code.as_bytes()[..4], &0x07230203u32.to_le_bytes());
    }
}

#[test]
fn all_borrowed_source_kinds_reject_foreign_sessions() {
    let global = GlobalSession::new().unwrap();
    let first = global.create_session(&SessionOptions::default()).unwrap();
    let second = global.create_session(&SessionOptions::default()).unwrap();
    let module = first
        .load_module_from_source(
            "foreign",
            "foreign.slang",
            r#"
        [shader("compute")][numthreads(1,1,1)] void main() {}
    "#,
        )
        .unwrap()
        .value;
    let entry = module.find_entry_point("main").unwrap();
    let component = first
        .compose_refs(&[module.as_component_ref(), entry.as_component_ref()])
        .unwrap()
        .value;
    for view in [
        module.as_component_ref(),
        entry.as_component_ref(),
        component.as_component_ref(),
    ] {
        assert!(matches!(
            second.compose_refs(&[view]),
            Err(Error::InvalidArgument(_))
        ));
    }
    assert!(matches!(
        first.compose_refs(&[]),
        Err(Error::InvalidArgument(_))
    ));
    // Keep the old empty-slice call type-inference compatible as well.
    assert!(matches!(first.compose(&[]), Err(Error::InvalidArgument(_))));
}
