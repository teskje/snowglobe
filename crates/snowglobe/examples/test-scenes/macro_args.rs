use std::time::Duration;

use snowglobe::__private::*;
use snowglobe::Sim;

fn get_scene(name: &str) -> &'static Scene {
    SCENES
        .iter()
        .find(|s| s.module == "test_scenes::macro_args" && s.name == name)
        .unwrap()
}

#[snowglobe::scene]
fn bare(_sim: Sim) {
    let scene = get_scene("bare");
    assert_eq!(
        scene.config,
        SceneConfig {
            simulation_duration: None,
            tick_duration: None,
            min_message_latency: None,
            max_message_latency: None,
            fail_rate: None,
            repair_rate: None,
        }
    );
}

#[snowglobe::scene(
    simulation_duration = "60s",
    tick_duration = "1ms",
    min_message_latency = "5ms",
    max_message_latency = "100ms"
)]
fn durations(_sim: Sim) {
    let scene = get_scene("durations");
    assert_eq!(
        scene.config,
        SceneConfig {
            simulation_duration: Some(Duration::from_secs(60)),
            tick_duration: Some(Duration::from_millis(1)),
            min_message_latency: Some(Duration::from_millis(5)),
            max_message_latency: Some(Duration::from_millis(100)),
            fail_rate: None,
            repair_rate: None,
        }
    );
}

#[snowglobe::scene(fail_rate = 0.1, repair_rate = 0.5)]
fn rates(_sim: Sim) {
    let scene = get_scene("rates");
    assert_eq!(
        scene.config,
        SceneConfig {
            simulation_duration: None,
            tick_duration: None,
            min_message_latency: None,
            max_message_latency: None,
            fail_rate: Some(0.1),
            repair_rate: Some(0.5),
        }
    );
}
