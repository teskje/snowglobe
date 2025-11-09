use snowglobe::Sim;
use snowglobe::cli::scene;

#[scene]
fn thread_spawn(_sim: Sim) {
    std::thread::spawn(|| {});
}

#[scene]
fn tokio_spawn_blocking(mut sim: Sim) {
    sim.client("test", async {
        tokio::task::spawn_blocking(|| {}).await?;
        Ok(())
    });
    sim.run().unwrap();
}
