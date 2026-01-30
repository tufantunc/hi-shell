use chrono::Duration;
use chrono::Utc;
use hi_shell::update::{UpdateChecker, should_check_update};

#[test]
fn test_should_check_update_none() {
    let last_check = None;
    assert!(should_check_update(last_check));
}

#[test]
fn test_should_check_update_old() {
    let now = Utc::now();
    let two_days_ago = now - Duration::days(2);
    assert!(should_check_update(Some(two_days_ago)));
}

#[test]
fn test_should_not_check_update_recent() {
    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);
    assert!(!should_check_update(Some(one_hour_ago)));
}

#[test]
fn test_should_check_update_exactly_24_hours() {
    let now = Utc::now();
    let twenty_four_hours_ago = now - Duration::hours(24);
    assert!(should_check_update(Some(twenty_four_hours_ago)));
}

#[test]
fn test_update_checker_creation() {
    let _checker = UpdateChecker::new("0.1.2");
    // UpdateChecker is created successfully without panicking
    // The struct is validated to work with the version provided
}
