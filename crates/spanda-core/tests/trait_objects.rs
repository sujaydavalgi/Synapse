use spanda_core::{check, run, RunOptions};

#[test]
fn trait_object_var_and_method_dispatch() {
    let source = r#"
trait Greeter {
  fn greet() -> Void;
}

robot R {
  actuator wheels: DifferentialDrive;

  agent Nav {
    plan { wheels.stop(); }
  }

  impl Greeter for Nav {
    fn greet() -> Void { wheels.stop(); }
  }

  behavior run() {
    let handler: dyn Greeter = Nav;
    handler.greet();
  }
}
"#;
    check(source).expect("type-check trait object");
    run(source, RunOptions::default()).expect("run trait object dispatch");
}

#[test]
fn trait_object_rejects_unimplemented_agent() {
    let source = r#"
trait Worker {
  fn work() -> Void;
}

robot R {
  agent Helper {
    plan { }
  }

  behavior run() {
    let w: dyn Worker = Helper;
    w.work();
  }
}
"#;
    let err = check(source).expect_err("missing impl should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("does not implement trait")),
        "got {:?}",
        err.diagnostics()
    );
}
