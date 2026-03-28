use hi_shell::config::Config;
use hi_shell::telemetry::Telemetry;

#[test]
fn test_telemetry_disabled_by_default() {
    let config = Config::default();
    assert!(!config.telemetry_enabled);

    let telemetry = Telemetry::new(&config);
    telemetry.track_event("test_event", serde_json::json!({"key": "value"}));
}

#[test]
fn test_telemetry_disabled_no_api_key() {
    let mut config = Config::default();
    config.telemetry_enabled = true;
    let telemetry = Telemetry::new(&config);

    telemetry.track_event("test_event", serde_json::json!({"key": "value"}));
}

#[test]
fn test_telemetry_creates_instance() {
    let mut config = Config::default();
    config.telemetry_enabled = true;
    let telemetry = Telemetry::new(&config);
    drop(telemetry);
}
