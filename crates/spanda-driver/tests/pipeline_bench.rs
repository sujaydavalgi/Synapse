//! Manual pipeline timing baseline (run with `cargo test -p spanda-driver -- --ignored pipeline_bench`).

use spanda_driver::check;
use std::time::Instant;

const SAMPLE: &str = r#"
robot Bench {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety { max_speed = 1.0 m/s; }

  behavior run() {
    let _ = lidar.read();
    wheels.stop();
  }
}
"#;

#[test]
#[ignore]
fn pipeline_bench_check_only() {
    let start = Instant::now();
    check(SAMPLE).expect("check");
    eprintln!(
        "pipeline check: {} ms",
        start.elapsed().as_millis()
    );
}
