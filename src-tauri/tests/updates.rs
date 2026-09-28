use bambu_filament_migrator::updates::{
    GithubRelease, RELEASE_PAGE_PREFIX, UpdateCheck, release_page, select_update,
};
use serde_json::json;

fn releases(value: serde_json::Value) -> Vec<GithubRelease> {
    serde_json::from_value(value).unwrap()
}

fn available(tag: &str, current: &str) -> UpdateCheck {
    UpdateCheck::Available {
        current: current.to_owned(),
        latest: tag.trim_start_matches('v').to_owned(),
        tag: tag.to_owned(),
        url: format!("{RELEASE_PAGE_PREFIX}{tag}"),
    }
}

#[test]
fn beta_builds_are_offered_the_newest_published_prerelease() {
    let listed = releases(json!([
        {"tag_name": "v0.11.0", "draft": true, "prerelease": false, "html_url": "https://github.com/RemindZ/Spool-Ledger/releases/tag/untagged-1"},
        {"tag_name": "v0.10.0", "draft": false, "prerelease": true, "html_url": "https://github.com/RemindZ/Spool-Ledger/releases/tag/v0.10.0", "assets": []},
        {"tag_name": "v0.9.0", "draft": false, "prerelease": true, "html_url": "https://github.com/RemindZ/Spool-Ledger/releases/tag/v0.9.0"}
    ]));

    assert_eq!(
        select_update("0.9.0", &listed).unwrap(),
        available("v0.10.0", "0.9.0")
    );
}

#[test]
fn equal_and_older_releases_report_the_build_as_current() {
    let listed = releases(json!([
        {"tag_name": "v0.9.0", "draft": false, "prerelease": true},
        {"tag_name": "v0.8.2", "draft": false, "prerelease": false}
    ]));

    assert_eq!(
        select_update("0.9.0", &listed).unwrap(),
        UpdateCheck::Current {
            current: "0.9.0".to_owned()
        }
    );
}

#[test]
fn tags_outside_the_release_contract_are_ignored() {
    let listed = releases(json!([
        {"tag_name": "latest", "draft": false, "prerelease": false},
        {"tag_name": "v1.0", "draft": false, "prerelease": false},
        {"tag_name": "v1.0.0-rc1", "draft": false, "prerelease": true},
        {"tag_name": "1.0.0", "draft": false, "prerelease": false},
        {"tag_name": "v01.0.0", "draft": false, "prerelease": false}
    ]));

    assert_eq!(
        select_update("0.9.0", &listed).unwrap(),
        UpdateCheck::Current {
            current: "0.9.0".to_owned()
        }
    );
}

#[test]
fn stable_builds_ignore_prereleases() {
    let listed = releases(json!([
        {"tag_name": "v1.3.0", "draft": false, "prerelease": true},
        {"tag_name": "v1.2.1", "draft": false, "prerelease": false}
    ]));

    assert_eq!(
        select_update("1.2.0", &listed).unwrap(),
        available("v1.2.1", "1.2.0")
    );
}

#[test]
fn versions_compare_numerically() {
    let listed = releases(json!([
        {"tag_name": "v0.9.10", "draft": false, "prerelease": false},
        {"tag_name": "v0.10.0", "draft": false, "prerelease": false}
    ]));

    assert_eq!(
        select_update("0.9.2", &listed).unwrap(),
        available("v0.10.0", "0.9.2")
    );
}

#[test]
fn an_invalid_running_version_is_an_error() {
    assert!(select_update("dev", &[]).is_err());
}

#[test]
fn release_pages_are_built_only_from_contract_tags() {
    assert_eq!(
        release_page("v1.2.3").unwrap(),
        "https://github.com/RemindZ/Spool-Ledger/releases/tag/v1.2.3"
    );
    for tag in ["v1.2.3/../../settings", "https://example.com", "v1.2", ""] {
        assert!(release_page(tag).is_err(), "{tag}");
    }
}

#[test]
#[ignore = "reads the public GitHub release list over the network"]
fn live_release_list_offers_the_published_beta_to_an_older_build() {
    let result = bambu_filament_migrator::updates::check("0.0.1").unwrap();
    match result {
        UpdateCheck::Available { url, .. } => assert!(url.starts_with(RELEASE_PAGE_PREFIX)),
        other => panic!("expected an available update, got {other:?}"),
    }
}
