//! Tests for the `#[scene]` macro.

mod common;

/// Run a test scene, asserting that it succeeds.
fn test_success(scene: &str) {
    let output = common::run_test_scene(scene);
    assert!(output.status.success(), "{output}");
}

macro_rules! test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let scene = concat!("macro_args::", stringify!($name));
            test_success(scene);
        }
    };
}

test!(bare);
test!(durations);
test!(rates);
