//! Live gRPC probe against a running Control Center (`SPANDA_GRPC_BIND`).
use spanda_api::grpc::spanda_v1::control_center_client::ControlCenterClient;
use spanda_api::grpc::spanda_v1::{
    DeviceIdRequest, DriftRequest, Empty, JsonBodyRequest, QueryRequest, ReadinessRequest,
    TrustPackageRequest,
};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

async fn connect(bind: &str) -> ControlCenterClient<Channel> {
    let channel = Channel::from_shared(format!("http://{bind}"))
        .expect("grpc url")
        .connect()
        .await
        .expect("grpc connect");
    ControlCenterClient::new(channel)
}

#[tokio::test]
async fn grpc_live_control_center_endpoints() {
    let Some(bind) = std::env::var("SPANDA_GRPC_BIND").ok() else {
        return;
    };
    let mut client = connect(&bind).await;
    let health = client
        .health(Empty {})
        .await
        .expect("health")
        .into_inner();
    assert_eq!(health.status, "ok");

    let tenant = client
        .get_tenant(Empty {})
        .await
        .expect("tenant")
        .into_inner();
    assert!(tenant.json.contains("tenant_id"));

    let backend = client
        .get_observability_backend(Empty {})
        .await
        .expect("observability backend")
        .into_inner();
    assert!(backend.json.contains("spanda-otel-collector"));

    let devices = client
        .list_devices(Empty {})
        .await
        .expect("list devices")
        .into_inner();
    assert!(devices.json.contains("\"devices\""));

    let agents = client
        .list_fleet_agents(Empty {})
        .await
        .expect("list fleet agents")
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
    assert!(openapi.json.contains("Spanda"));

    let health_summary = client
        .get_health_summary(Empty {})
        .await
        .expect("health summary")
        .into_inner();
    assert!(health_summary.json.contains("overall_status"));

    let metrics = client
        .get_otlp_metrics(Empty {})
        .await
        .expect("otlp metrics")
        .into_inner();
    assert!(metrics.json.contains("resourceMetrics"));

    let scorecard = client
        .get_executive_scorecard(Empty {})
        .await
        .expect("scorecard")
        .into_inner();
    assert!(scorecard.json.contains("scorecard"));

    let thread = client
        .query_digital_thread(QueryRequest {
            query: String::new(),
        })
        .await
        .expect("digital thread")
        .into_inner();
    assert!(thread.json.contains("digital_thread"));

    let discovery = client
        .discover_devices(QueryRequest {
            query: "transport=mdns".into(),
        })
        .await
        .expect("discover devices")
        .into_inner();
    assert!(discovery.json.contains("installed_packages"));

    let device = client
        .get_device(DeviceIdRequest {
            device_id: "gps-001".into(),
        })
        .await
        .expect("get device")
        .into_inner();
    assert!(device.json.contains("gps-001") || device.json.contains("device"));

    if let Ok(baseline_id) = std::env::var("SPANDA_GRPC_BASELINE_ID") {
        let drift = client
            .detect_drift(DriftRequest { baseline_id })
            .await
            .expect("drift")
            .into_inner();
        assert!(drift.json.contains("dimensions_checked"));
    }

    let robots = client
        .list_robots(Empty {})
        .await
        .expect("list robots")
        .into_inner();
    assert!(robots.json.contains("robots"));

    if let Ok(api_key) = std::env::var("SPANDA_API_KEY") {
        let mut ota_req = tonic::Request::new(JsonBodyRequest {
            body_json: r#"{"strategy":"canary","version":"smoke-1.0","dry_run":true}"#.into(),
        });
        ota_req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {api_key}")).expect("metadata"),
        );
        let plan = client
            .plan_ota(ota_req)
            .await
            .expect("plan ota")
            .into_inner();
        assert!(plan.json.contains("rollout"));

        let mut exec_req = tonic::Request::new(JsonBodyRequest {
            body_json: r#"{"strategy":"all","version":"smoke-1.0","dry_run":true}"#.into(),
        });
        exec_req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {api_key}")).expect("metadata"),
        );
        let execute = client
            .execute_ota(exec_req)
            .await
            .expect("execute ota")
            .into_inner();
        assert!(execute.json.contains("rollout"));

        let mut secrets_req = tonic::Request::new(Empty {});
        secrets_req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {api_key}")).expect("metadata"),
        );
        let secrets = client
            .list_secrets(secrets_req)
            .await
            .expect("list secrets")
            .into_inner();
        assert!(secrets.json.contains("secrets"));
    }
}
