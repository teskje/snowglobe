//! Tests that validate that code running in a simulation cannot break out.

mod common;

/// Run a test scene, asserting that it fails with the expected error.
fn test_containment(scene: &str, error: &str) {
    let output = common::run_test_scene(scene);
    assert!(!output.status.success(), "{output}");
    assert!(output.stderr.contains(error), "{output}");
}

#[test]
fn thread_spawn() {
    test_containment("containment::thread_spawn", "Operation not permitted");
}

#[test]
fn tokio_spawn_blocking() {
    test_containment(
        "containment::tokio_spawn_blocking",
        "Operation not permitted",
    );
}
