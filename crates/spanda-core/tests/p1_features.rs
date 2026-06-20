use spanda_core::{check, run, run_tests, RunOptions};

#[test]
fn serialize_json_round_trip() {
    let source = r#"
module telemetry;

export fn snapshot() -> Pose {
  return pose(x: 1.0 m, y: 2.0 m, theta: 0.0 rad);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let p = snapshot();
    let data = serialize(p, "json");
    let restored = deserialize(data, "json");
    let _ = restored;
    wheels.stop();
  }
}
"#;
    check(source).expect("serialize/deserialize should type-check");
    run(source, RunOptions::default()).expect("serialize should run");
}

#[test]
fn in_language_test_block_runs() {
    let source = r#"
module math;

export fn double(x: Int) -> Int {
  return x;
}

test "double returns input" {
  assert(true);
}
"#;
    let result = run_tests(source).expect("tests should execute");
    assert_eq!(result.passed, 1);
    assert_eq!(result.failed, 0);
}

#[test]
fn async_await_module_function() {
    let source = r#"
module maps;

export async fn get_map() -> Pose {
  return pose(x: 0.0 m, y: 0.0 m, theta: 0.0 rad);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let map = await get_map();
    let _ = map;
    wheels.stop();
  }
}
"#;
    check(source).expect("async/await should type-check");
    run(source, RunOptions::default()).expect("async/await should run");
}

#[test]
fn spawn_channel_select() {
    let source = r#"
module comm;

export fn ping() -> Int {
  return 1;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    send(ch, 42);
    select {
      recv(ch) => wheels.stop();
    };
    spawn ping();
  }
}
"#;
    check(source).expect("spawn/channel/select should type-check");
    run(source, RunOptions::default()).expect("concurrency primitives should run");
}
