use chrono::{Duration, Utc};
use hi_shell::update::{format_update_message, should_check_update, UpdateChecker, UpdateInfo};

#[test]
fn test_compare_major_update() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(checker.compare_versions("0.2.2", "1.0.0"));
}

#[test]
fn test_compare_minor_update() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(checker.compare_versions("0.2.2", "0.3.0"));
}

#[test]
fn test_compare_patch_update() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(checker.compare_versions("0.2.2", "0.2.3"));
}

#[test]
fn test_compare_same_version() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(!checker.compare_versions("0.2.2", "0.2.2"));
}

#[test]
fn test_compare_older_version() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(!checker.compare_versions("0.2.2", "0.2.1"));
}

#[test]
fn test_compare_invalid_current() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(checker.compare_versions("not-a-version", "1.0.0"));
}

#[test]
fn test_compare_invalid_latest() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(!checker.compare_versions("1.0.0", "not-a-version"));
}

#[test]
fn test_compare_both_invalid() {
    let checker = UpdateChecker::new("0.2.2").unwrap();
    assert!(!checker.compare_versions("invalid", "invalid"));
}

#[test]
fn test_format_update_message() {
    let info = UpdateInfo {
        current_version: "0.2.2".to_string(),
        latest_version: "0.3.0".to_string(),
        needs_update: true,
        release_url: "https://github.com/tufantunc/hi-shell/releases/tag/v0.3.0".to_string(),
        published_at: "2026-03-28T00:00:00Z".to_string(),
    };
    let msg = format_update_message(&info);
    assert!(msg.contains("0.2.2"));
    assert!(msg.contains("0.3.0"));
    assert!(msg.contains("https://github.com"));
}

#[test]
fn test_should_check_update_none() {
    assert!(should_check_update(None));
}

#[test]
fn test_should_check_update_old() {
    let two_days_ago = Utc::now() - Duration::days(2);
    assert!(should_check_update(Some(two_days_ago)));
}

#[test]
fn test_should_not_check_update_recent() {
    let one_hour_ago = Utc::now() - Duration::hours(1);
    assert!(!should_check_update(Some(one_hour_ago)));
}

#[test]
fn test_should_check_update_exactly_24_hours() {
    let twenty_four_hours_ago = Utc::now() - Duration::hours(24);
    assert!(should_check_update(Some(twenty_four_hours_ago)));
}
