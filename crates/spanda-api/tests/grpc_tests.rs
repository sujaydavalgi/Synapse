//! gRPC server smoke tests for Control Center.
use spanda_api::grpc::spanda_v1::control_center_client::ControlCenterClient;
use spanda_api::grpc::spanda_v1::{Empty, QueryRequest, ReadinessRequest, TrustPackageRequest};
use spanda_api::grpc::spanda_v1::{DeviceIdRequest, DeviceBodyRequest};
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

#[tokio::test]
async fn grpc_health_and_dashboard() {
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let grpc_bind = format!("127.0.0.1:{grpc_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(grpc_bind.clone()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(400));
    let mut client = connect(&grpc_bind).await;
    let health = client
        .health(Empty {})
        .await
        .expect("health rpc")
        .into_inner();
    assert_eq!(health.status, "ok");
    let dashboard = client
        .get_dashboard(Empty {})
        .await
        .expect("dashboard rpc")
        .into_inner();
    assert!(dashboard.json.contains("device_pool"));
}

#[tokio::test]
async fn grpc_expanded_endpoints_return_json() {
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let grpc_bind = format!("127.0.0.1:{grpc_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(grpc_bind.clone()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(400));
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
    let channel = Channel::from_shared(format!("http://{bind}"))
        .unwrap()
        .connect()
        .await
        .expect("grpc connect");
    ControlCenterClient::new(channel)
}

#[tokio::test]
async fn grpc_mutation_rbac_from_metadata() {
    let _guard = GRPC_TEST_LOCK.lock().unwrap();
    std::env::set_var("SPANDA_API_KEY", "grpc-rbac-test-key");
    let http_port = pick_port();
    let grpc_port = pick_port();
    let http_bind = format!("127.0.0.1:{http_port}");
    let grpc_bind = format!("127.0.0.1:{grpc_port}");
    let options = ControlCenterOptions {
        bind: http_bind,
        grpc_bind: Some(grpc_bind.clone()),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(400));
    let mut client = connect(&grpc_bind).await;

    let denied = client
        .plan_ota(tonic::Request::new(spanda_api::grpc::spanda_v1::JsonBodyRequest {
            body_json: r#"{"strategy":"canary","version":"1.0.0","dry_run":true}"#.into(),
        }))
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
    let warehouse = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spanda-config/tests/fixtures/warehouse/spanda.toml");
    let http_port = pick_port();
    let grpc_port = pick_port();
    let options = ControlCenterOptions {
        bind: format!("127.0.0.1:{http_port}"),
        grpc_bind: Some(format!("127.0.0.1:{grpc_port}")),
        config_path: Some(warehouse),
        once: true,
        timeout_ms: 500,
        ..Default::default()
    };
    let grpc_bind = options.grpc_bind.clone().unwrap();
    thread::spawn(move || {
        let _ = run_control_center_server(&options);
    });
    thread::sleep(Duration::from_millis(500));
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
