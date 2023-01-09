// Tests for calls to the /repos/{owner}/{repo}/events API.
mod mock_error;

use chrono::{DateTime, Utc};
use reqwest::header::CONTENT_TYPE;
use mock_error::setup_error_handler;
use octocrab::{etag::{EntityTag, Etagged}, models::events, Octocrab, Page};
use serde::{Deserialize, Serialize};
use url::Url;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use octocrab::models::repos::DeployKey;

async fn setup_api(template: ResponseTemplate) -> MockServer {
    let owner = "owner";
    let repo = "repo";
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/repos/{}/{}/keys", owner, repo)))
        .respond_with(template)
        .mount(&mock_server)
        .await;
    setup_error_handler(
        &mock_server,
        &format!("GET on /repo/{}/{}/keys was not received", owner, repo),
    )
        .await;
    mock_server
}

fn setup_octocrab(uri: &str) -> Octocrab {
    Octocrab::builder().base_url(uri).unwrap().build().unwrap()
}

const OWNER: &str = "owner";
const REPO: &str = "repo";

#[tokio::test]
async fn should_return_keys() {
    let deploy_keys = include_str!("resources/repo_deploy_keys.json");

    let template = ResponseTemplate::new(200)
        .set_body_string(deploy_keys)
        .insert_header(CONTENT_TYPE, "application/json; charset=utf-8");
    let mock_server = setup_api(template).await;
    let octo = setup_octocrab(&mock_server.uri());
    let repos = octo.repos(OWNER.to_owned(), REPO.to_owned());
    let result = repos.list_deploy_keys().send().await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );

    let keys = result.unwrap();
    assert_eq!(1, keys.len());
    assert_eq!(DeployKey{
        id: 243243243,
        key: "ssh-ed25519 BBBBC3NzaC1lZDI1NTE5BBBBIGMqhDgPeBG9oVxbo5jfoidh7h96m1am8HJJA76V+jPJ".to_string(),
        url: Url::parse("https://api.github.com/repos/org/repo/keys/243243243").unwrap(),
        title: "MyDeployKey".to_string(),
        verified: true,
        created_at: DateTime::parse_from_rfc3339("2023-01-06T11:56:18Z").unwrap().with_timezone(&Utc),
        read_only: true,
        added_by: Some("jdoe".to_string()),
        last_used: Some(DateTime::parse_from_rfc3339("2023-01-06T11:58:49Z").unwrap().with_timezone(&Utc))
    }, keys[0]);

}


#[tokio::test]
async fn should_return_no_keys_for_404() {
    let deploy_keys = "[]";

    let template = ResponseTemplate::new(200)
        .set_body_string(deploy_keys)
        .insert_header(CONTENT_TYPE, "application/json; charset=utf-8");
    let mock_server = setup_api(template).await;
    let octo = setup_octocrab(&mock_server.uri());
    let repos = octo.repos(OWNER.to_owned(), REPO.to_owned());
    let result = repos.list_deploy_keys().send().await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );

    let keys = result.unwrap();
    assert_eq!(0, keys.len());

}
