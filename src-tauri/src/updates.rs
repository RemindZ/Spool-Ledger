//! Manual update notification: read published GitHub releases and report a
//! newer version. Nothing is downloaded or installed; the user opens the
//! release page in their browser.

use crate::AppError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const RELEASES_API: &str =
    "https://api.github.com/repos/RemindZ/Spool-Ledger/releases?per_page=30";
pub const RELEASE_PAGE_PREFIX: &str = "https://github.com/RemindZ/Spool-Ledger/releases/tag/";

#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateCheck {
    Available {
        current: String,
        latest: String,
        tag: String,
        url: String,
    },
    Current {
        current: String,
    },
}

type Version = (u64, u64, u64);

/// Parses `MAJOR.MINOR.PATCH` with plain decimal parts and no leading zeros.
fn parse_version(value: &str) -> Option<Version> {
    let mut parts = value.split('.');
    let mut next = || {
        let part = parts.next()?;
        let valid = !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'));
        valid.then(|| part.parse().ok()).flatten()
    };
    let version = (next()?, next()?, next()?);
    parts.next().is_none().then_some(version)
}

/// Release tags follow the release contract: `vMAJOR.MINOR.PATCH`.
fn parse_tag(tag: &str) -> Option<Version> {
    parse_version(tag.strip_prefix('v')?)
}

pub fn release_page(tag: &str) -> Result<String, AppError> {
    parse_tag(tag)
        .map(|_| format!("{RELEASE_PAGE_PREFIX}{tag}"))
        .ok_or_else(|| AppError::UpdateCheck(format!("not a release tag: {tag}")))
}

/// Picks the highest published release above `current`. Drafts are ignored.
/// Pre-releases count while the running build is 0.x, because public betas
/// ship as pre-releases; stable builds only see stable releases.
pub fn select_update(current: &str, releases: &[GithubRelease]) -> Result<UpdateCheck, AppError> {
    let running = parse_version(current)
        .ok_or_else(|| AppError::UpdateCheck(format!("invalid running version: {current}")))?;
    let include_prereleases = running.0 == 0;
    let newest = releases
        .iter()
        .filter(|release| !release.draft && (include_prereleases || !release.prerelease))
        .filter_map(|release| parse_tag(&release.tag_name).map(|version| (version, release)))
        .filter(|(version, _)| *version > running)
        .max_by_key(|(version, _)| *version);
    Ok(match newest {
        Some(((major, minor, patch), release)) => UpdateCheck::Available {
            current: current.to_owned(),
            latest: format!("{major}.{minor}.{patch}"),
            tag: release.tag_name.clone(),
            url: release_page(&release.tag_name)?,
        },
        None => UpdateCheck::Current {
            current: current.to_owned(),
        },
    })
}

/// Reads the public release list over the operating system TLS stack.
/// One unauthenticated request per call.
pub async fn check(current: &str) -> Result<UpdateCheck, AppError> {
    let failed = |error: reqwest::Error| AppError::UpdateCheck(error.to_string());
    let client = reqwest::Client::builder()
        .user_agent(format!("SpoolLedger/{current}"))
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(failed)?;
    let body = client
        .get(RELEASES_API)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(failed)?
        .error_for_status()
        .map_err(failed)?
        .text()
        .await
        .map_err(failed)?;
    let releases: Vec<GithubRelease> = serde_json::from_str(&body)
        .map_err(|error| AppError::UpdateCheck(format!("unexpected release list: {error}")))?;
    select_update(current, &releases)
}
