//! Per-thread counters for wrapper costs, excluded from production builds.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Costs {
    pub allocations: usize,
    pub add_refs: usize,
    pub releases: usize,
}
thread_local! {
    static COSTS: Cell<Option<Costs>> = const { Cell::new(None) };
}
fn record(update: impl FnOnce(&mut Costs)) {
    let _ = COSTS.try_with(|costs| {
        if let Some(mut value) = costs.get() {
            update(&mut value);
            costs.set(Some(value));
        }
    });
}
pub(crate) fn record_add_ref() {
    record(|costs| costs.add_refs += 1);
}
pub(crate) fn record_release() {
    record(|costs| costs.releases += 1);
}

pub(crate) fn measure<T>(call: impl FnOnce() -> T) -> (T, Costs) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            COSTS.with(|costs| costs.set(None));
        }
    }
    COSTS.with(|costs| {
        assert!(costs.get().is_none(), "cost measurements cannot nest");
        costs.set(Some(Costs::default()));
    });
    let _reset = Reset;
    let result = call();
    let costs = COSTS.with(|costs| costs.replace(None)).unwrap();
    (result, costs)
}

struct CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(|costs| costs.allocations += 1);
        // SAFETY: forwarding the allocator contract unchanged to System.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarding the allocator contract unchanged to System.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(|costs| costs.allocations += 1);
        // SAFETY: forwarding the allocator contract unchanged to System.
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
