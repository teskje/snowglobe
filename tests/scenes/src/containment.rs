pub fn thread_spawn(_sim: snowglobe::Sim) {
    std::thread::spawn(|| {});
}

pub fn tokio_spawn_blocking(mut sim: snowglobe::Sim) {
    sim.client("test", async {
        tokio::task::spawn_blocking(|| {}).await?;
        Ok(())
    });
    sim.run().unwrap();
}
