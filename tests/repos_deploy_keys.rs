// Tests for calls to the /repos/{owner}/{repo}/keys API.
mod mock_error;

use chrono::{DateTime, Utc};
use reqwest::header::CONTENT_TYPE;
use octocrab::{etag::{EntityTag, Etagged}, models::events, Octocrab, Page};
use url::Url;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use octocrab::models::repos::DeployKey;


const OWNER: &str = "owner";
const REPO: &str = "repo";

async fn setup_list_api(template: ResponseTemplate) -> MockServer {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/repos/{}/{}/keys", OWNER, REPO)))
        .respond_with(template)
        .named("GET key list")
        .expect(1)
        .mount(&mock_server)
        .await;
    mock_server
}

async fn setup_get_key_api(id: &str, template: ResponseTemplate) -> MockServer {

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/repos/{}/{}/keys/{}", OWNER, REPO, id)))
        .respond_with(template)
        .named("GET key by id")
        .expect(1)
        .mount(&mock_server)
        .await;
    mock_server
}

fn setup_octocrab(uri: &str) -> Octocrab {
    Octocrab::builder().base_url(uri).unwrap().build().unwrap()
}


#[tokio::test]
async fn list_deploy_keys_should_return_keys() {
    let deploy_keys = include_str!("resources/repo_deploy_keys.json");

    let template = ResponseTemplate::new(200)
        .set_body_string(deploy_keys)
        .insert_header(CONTENT_TYPE, "application/json; charset=utf-8");
    let mock_server = setup_list_api(template).await;
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
    assert_eq!(expected_key(), keys[0]);

}


#[tokio::test]
async fn list_deploy_keys_should_return_no_keys_for_404() {
    let deploy_keys = "[]";

    let template = ResponseTemplate::new(200)
        .set_body_string(deploy_keys)
        .insert_header(CONTENT_TYPE, "application/json; charset=utf-8");
    let mock_server = setup_list_api(template).await;
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



#[tokio::test]
async fn get_key_should_return_key() {
    let deploy_keys = include_str!("resources/repo_deploy_key.json");

    let template = ResponseTemplate::new(200)
        .set_body_string(deploy_keys)
        .insert_header(CONTENT_TYPE, "application/json; charset=utf-8");
    let mock_server = setup_get_key_api("243243243", template).await;
    let octo = setup_octocrab(&mock_server.uri());
    let repos = octo.repos(OWNER.to_owned(), REPO.to_owned());
    let result = repos.get_deploy_key("243243243").await;
    assert!(
        result.is_ok(),
        "expected successful result, got error: {:#?}",
        result
    );

    let key = result.unwrap();
    assert_eq!(expected_key(), key);
}


#[tokio::test]
async fn get_key_should_handle_404() {
    let template = ResponseTemplate::new(404);
    let mock_server = setup_get_key_api("243243243", template).await;
    let octo = setup_octocrab(&mock_server.uri());
    let repos = octo.repos(OWNER.to_owned(), REPO.to_owned());
    let result = repos.get_deploy_key("243243243").await;
    assert!(result.is_err());
}



fn expected_key() -> DeployKey {
    DeployKey{
        id: 243243243,
        key: "ssh-ed25519 BBBBC3NzaC1lZDI1NTE5BBBBIGMqhDgPeBG9oVxbo5jfoidh7h96m1am8HJJA76V+jPJ".to_string(),
        url: Url::parse("https://api.github.com/repos/org/repo/keys/243243243").unwrap(),
        title: "MyDeployKey".to_string(),
        verified: true,
        created_at: DateTime::parse_from_rfc3339("2023-01-06T11:56:18Z").unwrap().with_timezone(&Utc),
        read_only: true,
        added_by: Some("jdoe".to_string()),
        last_used: Some(DateTime::parse_from_rfc3339("2023-01-06T11:58:49Z").unwrap().with_timezone(&Utc))
    }
}