use snowglobe::Sim;
use snowglobe::cli::scene;

#[scene]
pub fn thread_spawn(_sim: Sim) {
    std::thread::spawn(|| {});
}

#[scene]
pub fn tokio_spawn_blocking(mut sim: Sim) {
    sim.client("test", async {
        tokio::task::spawn_blocking(|| {}).await?;
        Ok(())
    });
    sim.run().unwrap();
}
