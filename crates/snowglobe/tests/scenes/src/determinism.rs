use std::collections::HashSet;
use std::time::Duration;

use snowglobe::Sim;
use snowglobe::cli::scene;
use tokio::time::sleep;

#[scene]
pub fn random_numbers(_sim: Sim) {
    print!("{},", rand::random::<u8>());
    print!("{},", rand::random::<u16>());
    print!("{},", rand::random::<u32>());
    print!("{},", rand::random::<u64>());
    print!("{},", rand::random::<u128>());
}

#[scene]
pub fn select_branch(mut sim: Sim) {
    sim.client("test", async {
        for _ in 0..10 {
            let branch = tokio::select! {
                _ = async {} => 1,
                _ = async {} => 2,
                _ = async {} => 3,
            };
            print!("{branch}");
        }
        Ok(())
    });
    sim.run().unwrap();
}

#[scene]
pub fn hashset_order(_sim: Sim) {
    let set: HashSet<_> = (0..10).collect();
    let vec: Vec<_> = set.into_iter().collect();
    print!("{vec:?}");
}

#[scene]
pub fn tokio_time(mut sim: Sim) {
    use tokio::time::Instant;

    sim.client("test", async {
        sleep(Duration::from_secs(1)).await;
        print!("{:?},", Instant::now());
        sleep(Duration::from_millis(1)).await;
        print!("{:?},", Instant::now());
        sleep(Duration::from_nanos(1)).await;
        print!("{:?},", Instant::now());
        Ok(())
    });
    sim.run().unwrap();
}

#[scene]
pub fn std_time(mut sim: Sim) {
    use std::time::{Instant, SystemTime};

    sim.client("test", async {
        sleep(Duration::from_secs(1)).await;
        print!("{:?} {:?},", Instant::now(), SystemTime::now());
        sleep(Duration::from_millis(1)).await;
        print!("{:?} {:?},", Instant::now(), SystemTime::now());
        sleep(Duration::from_nanos(1)).await;
        print!("{:?} {:?},", Instant::now(), SystemTime::now());
        Ok(())
    });
    sim.run().unwrap();
}

#[scene]
pub fn uuid(_sim: Sim) {
    print!("{}", uuid::Uuid::now_v7());
}
