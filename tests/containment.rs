//! Tests that validate that code running in a simulation cannot break out.

#[test]
#[should_panic(expected = "Operation not permitted")]
fn panic_on_thread_spawn() {
    let cfg = snowglobe::Config::default();
    snowglobe::simulation(cfg, |_sim| {
        std::thread::spawn(|| {}).join().unwrap();
    })
    .unwrap();
}

#[test]
#[should_panic(expected = "a spawned task panicked")]
fn panic_on_tokio_spawn_blocking() {
    let cfg = snowglobe::Config::default();
    snowglobe::simulation(cfg, |mut sim| {
        sim.client("test", async {
            tokio::task::spawn_blocking(|| {}).await?;
            Ok(())
        });
        sim.run().unwrap();
    })
    .unwrap();
}
