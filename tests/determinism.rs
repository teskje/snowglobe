//! Tests that validate that code running in a simulation is deterministic.

mod common;

/// Run a test scene N times, asserting that it produces the same output each time.
fn test_determinism(scene: &str) {
    const N: u32 = 10;

    let mut expected: Option<String> = None;

    for _ in 0..N {
        let output = common::run_test_scene(scene);
        assert!(output.status.success(), "{output}");

        if let Some(expected) = &expected {
            assert_eq!(output.stdout, *expected);
        } else {
            expected = Some(output.stdout);
        }
    }
}

#[test]
fn random_numbers() {
    test_determinism("determinism::random_numbers");
}

#[test]
fn select_branch() {
    test_determinism("determinism::select_branch");
}

#[test]
fn hashset_order() {
    test_determinism("determinism::hashset_order");
}

#[test]
fn tokio_time() {
    test_determinism("determinism::tokio_time");
}

#[test]
fn std_time() {
    test_determinism("determinism::std_time");
}
