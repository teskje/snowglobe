use std::collections::HashSet;
use std::time::Duration;

use snowglobe::Sim;
use tokio::time::sleep;

#[snowglobe::scene]
fn random_numbers(_sim: Sim) {
    print!("{},", rand::random::<u8>());
    print!("{},", rand::random::<u16>());
    print!("{},", rand::random::<u32>());
    print!("{},", rand::random::<u64>());
    print!("{},", rand::random::<u128>());
}

#[snowglobe::scene]
fn openssl_rand_bytes(_sim: Sim) {
    let mut buf = [0; 10];
    openssl::rand::rand_bytes(&mut buf).unwrap();
    print!("{buf:?}");
}

#[snowglobe::scene]
fn select_branch(mut sim: Sim) {
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

#[snowglobe::scene]
fn hashset_order(_sim: Sim) {
    let set: HashSet<_> = (0..10).collect();
    let vec: Vec<_> = set.into_iter().collect();
    print!("{vec:?}");
}

#[snowglobe::scene]
fn tokio_time(mut sim: Sim) {
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

#[snowglobe::scene]
fn std_time(mut sim: Sim) {
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

#[snowglobe::scene]
fn uuid(_sim: Sim) {
    print!("{}", uuid::Uuid::now_v7());
}

#[snowglobe::scene]
fn heap_address(_sim: Sim) {
    let v = vec![1];
    print!("{:?}", v.as_ptr());
}

#[snowglobe::scene]
fn heap_address_ffi(_sim: Sim) {
    let ptr = unsafe { libc::malloc(1) };
    print!("{:?}", ptr);
}
