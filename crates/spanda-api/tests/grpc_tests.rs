//! gRPC server smoke tests for Control Center.
use spanda_api::grpc::spanda_v1::control_center_client::ControlCenterClient;
use spanda_api::grpc::spanda_v1::{DeviceBodyRequest, DeviceIdRequest, EntityBodyRequest, EntityIdRequest};
use spanda_api::grpc::spanda_v1::{Empty, QueryRequest, ReadinessRequest, TrustPackageRequest};
use spanda_api::{run_control_center_server, ControlCenterOptions};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tonic::transport::Channel;

static GRPC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn pick_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral")
        .local_addr()
        .expect("local addr")
        .port()
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).expect("create temp config dir");
    for entry in std::fs::read_dir(src).expect("read config source") {
        let entry = entry.expect("config entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy config file");
        }
    }
}

#[tokio::test]
async fn grpc_health_and_dashboard() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let mut client = connect(&grpc_bind).await;
    let health = client
        .health(Empty {})
        .await
        .expect("health rpc")
        .into_inner();
    assert!(health.status.starts_with("ok"));
    let dashboard = client
        .get_dashboard(Empty {})
        .await
        .expect("dashboard rpc")
        .into_inner();
    assert!(dashboard.json.contains("device_pool"));
}

#[tokio::test]
async fn grpc_expanded_endpoints_return_json() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let mut client = connect(&grpc_bind).await;

    let devices = client
        .list_devices(Empty {})
        .await
        .expect("devices")
        .into_inner();
    assert!(devices.json.contains("\"devices\""));

    let agents = client
        .list_fleet_agents(Empty {})
        .await
        .expect("agents")
        .into_inner();
    assert!(agents.json.contains("\"agents\""));

    let readiness = client
        .evaluate_readiness(ReadinessRequest {
            body_json: String::new(),
        })
        .await
        .expect("readiness")
        .into_inner();
    assert!(readiness.json.contains("mission_ready"));

    let sre = client
        .get_sre_summary(Empty {})
        .await
        .expect("sre")
        .into_inner();
    assert!(sre.json.contains("availability_percent"));

    let trust = client
        .get_trust_package(TrustPackageRequest {
            package_name: "spanda-mqtt".into(),
        })
        .await
        .expect("trust")
        .into_inner();
    assert!(trust.json.contains("trust"));

    let openapi = client
        .get_open_api(Empty {})
        .await
        .expect("openapi")
        .into_inner();
    assert!(openapi.json.contains("openapi"));

    let health_summary = client
        .get_health_summary(Empty {})
        .await
        .expect("health summary")
        .into_inner();
    assert!(health_summary.json.contains("overall_status"));

    let assurance = client
        .get_assurance_summary(Empty {})
        .await
        .expect("assurance")
        .into_inner();
    assert!(assurance.json.contains("loaded"));

    let diagnosis = client
        .get_diagnosis_summary(Empty {})
        .await
        .expect("diagnosis")
        .into_inner();
    assert!(diagnosis.json.contains("loaded"));

    let ota = client
        .get_ota_status(Empty {})
        .await
        .expect("ota")
        .into_inner();
    assert!(ota.json.contains("version"));

    let metrics = client
        .get_otlp_metrics(Empty {})
        .await
        .expect("otlp metrics")
        .into_inner();
    assert!(metrics.json.contains("resourceMetrics"));

    let discovery = client
        .discover_devices(QueryRequest {
            query: "transport=mdns".into(),
        })
        .await
        .expect("discover devices")
        .into_inner();
    assert!(discovery.json.contains("discovery"));

    let device_tree = client
        .get_device_tree(Empty {})
        .await
        .expect("device tree")
        .into_inner();
    assert!(device_tree.json.contains("loaded"));

    let traces = client
        .get_observability_traces(Empty {})
        .await
        .expect("observability traces")
        .into_inner();
    assert!(traces.json.contains("traces"));

    let otlp_traces = client
        .get_otlp_traces(Empty {})
        .await
        .expect("otlp traces")
        .into_inner();
    assert!(otlp_traces.json.contains("resourceSpans"));

    let rbac = client
        .get_rbac_matrix(Empty {})
        .await
        .expect("rbac matrix")
        .into_inner();
    assert!(rbac.json.contains("matrix"));
}

async fn connect(bind: &str) -> ControlCenterClient<Channel> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match Channel::from_shared(format!("http://{bind}"))
            .unwrap()
            .connect()
            .await
        {
            Ok(channel) => return ControlCenterClient::new(channel),
            Err(error) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let _ = &error;
            }
            Err(error) => panic!("grpc connect: {error:?}"),
        }
    }
}

fn spawn_control_center(options: ControlCenterOptions) -> String {
    let grpc_bind = options.grpc_bind.clone().expect("grpc bind");
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    grpc_bind
}

#[tokio::test]
async fn grpc_mutation_rbac_from_metadata() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_API_KEY", "grpc-rbac-test-key");
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let mut client = connect(&grpc_bind).await;

    let denied = client
        .plan_ota(tonic::Request::new(
            spanda_api::grpc::spanda_v1::JsonBodyRequest {
                body_json: r#"{"strategy":"canary","version":"1.0.0","dry_run":true}"#.into(),
            },
        ))
        .await
        .expect("plan ota without auth")
        .into_inner();
    assert!(denied.json.contains("unauthorized") || denied.json.contains("Unauthorized"));

    let mut authed = tonic::Request::new(spanda_api::grpc::spanda_v1::JsonBodyRequest {
        body_json: r#"{"strategy":"canary","version":"1.0.0","dry_run":true}"#.into(),
    });
    authed.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-rbac-test-key").unwrap(),
    );
    let allowed = client
        .plan_ota(authed)
        .await
        .expect("plan ota with bearer")
        .into_inner();
    assert!(allowed.json.contains("rollout"));
}

#[tokio::test]
async fn grpc_device_subresources_with_warehouse_config() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_API_KEY", "grpc-device-test-key");
    let warehouse_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spanda-config/tests/fixtures/warehouse");
    let temp_dir = tempfile::tempdir().expect("temp config dir");
    copy_dir_all(&warehouse_src, temp_dir.path());
    let http_port = pick_port();
    let grpc_port = pick_port();
    let options = ControlCenterOptions {
        bind: format!("127.0.0.1:{http_port}"),
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        config_path: Some(temp_dir.path().to_path_buf()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let _temp_dir = temp_dir;
    let mut client = connect(&grpc_bind).await;

    let device = client
        .get_device(DeviceIdRequest {
            device_id: "gps-001".into(),
        })
        .await
        .expect("get device")
        .into_inner();
    assert!(device.json.contains("gps-001"));

    let mut alert_req = tonic::Request::new(Empty {});
    alert_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let alert = client
        .test_alert(alert_req)
        .await
        .expect("test alert")
        .into_inner();
    assert!(alert.json.contains("Control Center alert test"));

    let mut snapshot_req = tonic::Request::new(spanda_api::grpc::spanda_v1::JsonBodyRequest {
        body_json: r#"{"label":"grpc-snapshot"}"#.into(),
    });
    snapshot_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let snapshot = client
        .save_config_snapshot(snapshot_req)
        .await
        .expect("save snapshot")
        .into_inner();
    assert!(snapshot.json.contains("snapshot"));

    let mut assign_req = tonic::Request::new(DeviceBodyRequest {
        device_id: "gps-001".into(),
        body_json: r#"{"robot_id":"rover-001","logical_name":"gps"}"#.into(),
    });
    assign_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let assign = client
        .device_assign(assign_req)
        .await
        .expect("device assign")
        .into_inner();
    assert!(assign.json.contains("result") || assign.json.contains("ok"));
}

#[tokio::test]
async fn grpc_sdk_program_and_entity_endpoints() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let mut client = connect(&grpc_bind).await;

    let entities = client
        .list_entities(Empty {})
        .await
        .expect("list entities")
        .into_inner();
    assert!(entities.json.contains("entities"));

    let readiness = client
        .evaluate_program_readiness(spanda_api::grpc::spanda_v1::JsonBodyRequest {
            body_json: r#"{"file":"examples/robotics/rover.sd"}"#.into(),
        })
        .await
        .expect("program readiness")
        .into_inner();
    assert!(
        readiness.json.contains("report") || readiness.json.contains("error"),
        "unexpected body: {}",
        readiness.json
    );
}

#[tokio::test]
async fn grpc_entity_graph_and_mutations_with_warehouse_config() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_API_KEY", "grpc-device-test-key");
    let warehouse_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spanda-config/tests/fixtures/warehouse");
    let temp_dir = tempfile::tempdir().expect("temp config dir");
    copy_dir_all(&warehouse_src, temp_dir.path());
    let program = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples/showcase/compliance/defense_rover.sd");
    let http_port = pick_port();
    let grpc_port = pick_port();
    let options = ControlCenterOptions {
        bind: format!("127.0.0.1:{http_port}"),
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        config_path: Some(temp_dir.path().to_path_buf()),
        program_path: Some(program),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = spawn_control_center(options);
    let _temp_dir = temp_dir;
    let mut client = connect(&grpc_bind).await;

    let graph = client
        .get_entity_graph(Empty {})
        .await
        .expect("entity graph")
        .into_inner();
    assert!(graph.json.contains("graph"));

    let trace = client
        .get_entity_traceability(QueryRequest {
            query: "entity_id=rover-001".into(),
        })
        .await
        .expect("entity traceability")
        .into_inner();
    assert!(trace.json.contains("traceability"));

    let mut verify_req = tonic::Request::new(EntityBodyRequest {
        entity_id: "rover-001".into(),
        body_json: "{}".into(),
    });
    verify_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let verified = client
        .verify_entity(verify_req)
        .await
        .expect("verify entity")
        .into_inner();
    assert!(verified.json.contains("entity_id"));

    let health = client
        .get_entity_health(EntityIdRequest {
            entity_id: "rover-001".into(),
        })
        .await
        .expect("entity health")
        .into_inner();
    assert!(health.json.contains("health_status"));

    let trust = client
        .get_entity_trust(EntityIdRequest {
            entity_id: "rover-001".into(),
        })
        .await
        .expect("entity trust")
        .into_inner();
    assert!(trust.json.contains("trust_status"));

    let mut register_req = tonic::Request::new(spanda_api::grpc::spanda_v1::JsonBodyRequest {
        body_json: r#"{
            "id": "grpc-smoke-bay",
            "entity_type": "calibration_station",
            "display_name": "gRPC Smoke Bay",
            "parent_id": "warehouse-a",
            "capabilities": ["calibrate"]
        }"#
        .into(),
    });
    register_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let registered = client
        .register_entity(register_req)
        .await
        .expect("register entity")
        .into_inner();
    assert!(registered.json.contains("grpc-smoke-bay"));

    let mut tag_req = tonic::Request::new(EntityBodyRequest {
        entity_id: "grpc-smoke-bay".into(),
        body_json: r#"{"add":["grpc-smoke"]}"#.into(),
    });
    tag_req.metadata_mut().insert(
        "authorization",
        tonic::metadata::MetadataValue::try_from("Bearer grpc-device-test-key").unwrap(),
    );
    let tagged = client
        .tag_entity(tag_req)
        .await
        .expect("tag entity")
        .into_inner();
    assert!(tagged.json.contains("grpc-smoke"));
}
