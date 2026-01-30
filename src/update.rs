use crate::error::{HiShellError, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    published_at: String,
}

pub struct UpdateChecker {
    client: Client,
    current_version: String,
    repo_owner: String,
    repo_name: String,
}

impl UpdateChecker {
    pub fn new(current_version: &str) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self {
            client,
            current_version: current_version.to_string(),
            repo_owner: "tufantunc".to_string(),
            repo_name: "hi-shell".to_string(),
        })
    }

    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "hi-shell")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HiShellError::UpdateCheck(
                "Failed to fetch release information".to_string(),
            ));
        }

        let release: GitHubRelease = response.json().await?;

        let latest_version = release.tag_name.trim_start_matches('v').to_string();
        let needs_update = self.compare_versions(&self.current_version, &latest_version);

        Ok(UpdateInfo {
            current_version: self.current_version.clone(),
            latest_version,
            needs_update,
            release_url: release.html_url,
            published_at: release.published_at,
        })
    }

    fn compare_versions(&self, current: &str, latest: &str) -> bool {
        let current_ver = Version::parse(current).unwrap_or_else(|_| {
            Version::new(
                current
                    .split('.')
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                current
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                current
                    .split('.')
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            )
        });

        let latest_ver = Version::parse(latest).unwrap_or_else(|_| {
            Version::new(
                latest
                    .split('.')
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                latest
                    .split('.')
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                latest
                    .split('.')
                    .nth(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            )
        });

        latest_ver > current_ver
    }
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
    pub release_url: String,
    pub published_at: String,
}

pub fn should_check_update(last_check: Option<DateTime<Utc>>) -> bool {
    match last_check {
        None => true,
        Some(last) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(last);
            duration.num_hours() >= 24
        }
    }
}

pub fn format_update_message(info: &UpdateInfo) -> String {
    format!(
        "\n{} {} → {}  \n{}",
        "🚀 New version available!".yellow().bold(),
        info.current_version.cyan(),
        info.latest_version.green().bold(),
        info.release_url.dimmed()
    )
}
