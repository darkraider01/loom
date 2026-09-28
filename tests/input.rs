#![deny(warnings, rust_2018_idioms)]

use loom::model::Builder;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::sync::Arc;
use loom::thread;
use std::sync::Arc as StdArc;

#[test]
fn explores_multiple_executions() {
    let calls = StdArc::new(std::sync::atomic::AtomicUsize::new(0));
    let recorded = calls.clone();
    Builder::new().check(move || {
        recorded.fetch_add(1, Ordering::Relaxed);
        let value = Arc::new(AtomicUsize::new(0));
        let writer = value.clone();
        let child = thread::spawn(move || writer.store(1, Ordering::Relaxed));
        value.load(Ordering::Relaxed);
        child.join().unwrap();
    });
    assert!(calls.load(Ordering::Relaxed) > 1);
}

#[test]
#[should_panic(expected = "model failure")]
fn propagates_panics() {
    Builder::new().check(|| panic!("model failure"));
}

#[test]
#[should_panic(expected = "maximum number of branches")]
fn enforces_branch_limit() {
    let mut builder = Builder::new();
    builder.max_branches = 10;
    builder.check(|| loop {
        thread::yield_now();
    });
}

#[test]
#[should_panic(expected = "leaked")]
fn detects_leaks() {
    Builder::new().check(|| {
        std::mem::forget(Arc::new(AtomicUsize::new(0)));
    });
}
