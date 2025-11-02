//! Tests that validate that code running in a simulation cannot break out.

mod common;

/// Run a test scene, asserting that it fails with the expected error.
fn test_containment(scene: &str, error: &str) {
    let output = common::run_test_scene(scene);
    assert!(!output.status.success(), "{output}");
    assert!(output.stderr.contains(error), "{output}");
}

macro_rules! test {
    ($name:ident, $error:literal) => {
        #[test]
        fn $name() {
            let scene = concat!("containment::", stringify!($name));
            test_containment(scene, $error);
        }
    };
}

test!(thread_spawn, "Operation not permitted");
test!(tokio_spawn_blocking, "Operation not permitted");
