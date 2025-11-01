use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::time::sleep;

/// Run a test case under simulation N times, asserting that it produces the same result each time.
fn test<F, Fut, R>(case: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = R> + Send + 'static,
    R: fmt::Debug + PartialEq + Send + 'static,
{
    const N: u32 = 10;

    let mut expected: Option<R> = None;

    for _ in 0..N {
        let cfg = snowglobe::Config::default();
        let fut = case();
        let (tx, rx) = oneshot::channel();

        snowglobe::simulation(cfg, |mut sim| {
            sim.client("test", async {
                let result = fut.await;
                tx.send(result).unwrap();
                Ok(())
            });
            sim.run().unwrap();
        });

        let result = rx.blocking_recv().unwrap();
        if let Some(expected) = &expected {
            assert_eq!(result, *expected);
        } else {
            expected = Some(result);
        }
    }
}

#[test]
fn deterministic_rand() {
    test(|| async { rand::random::<u64>() });
}

#[test]
fn deterministic_select() {
    test(|| async {
        tokio::select! {
            _ = async {} => 1,
            _ = async {} => 2,
            _ = async {} => 3,
            _ = async {} => 4,
            _ = async {} => 5,
        }
    });
}

#[test]
fn deterministic_hashset() {
    test(|| async {
        let set: HashSet<_> = (0..10).collect();
        let vec: Vec<_> = set.into_iter().collect();
        vec
    });
}

#[test]
fn deterministic_tokio_time() {
    test(|| async {
        sleep(Duration::from_secs(1)).await;
        tokio::time::Instant::now()
    });
}

#[test]
fn deterministic_std_time() {
    test(|| async {
        sleep(Duration::from_secs(1)).await;
        (std::time::Instant::now(), std::time::SystemTime::now())
    });
}
