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

macro_rules! test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let scene = concat!("determinism::", stringify!($name));
            test_determinism(scene);
        }
    };
}

test!(random_numbers);
test!(select_branch);
test!(hashset_order);
test!(tokio_time);
test!(std_time);
test!(uuid);
test!(heap_address);
test!(heap_address_ffi);
test!(openssl_rand_bytes);
