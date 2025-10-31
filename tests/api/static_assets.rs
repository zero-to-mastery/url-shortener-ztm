use crate::helpers::{TestApp, spawn_app};

use std::fs;

async fn assert_asset_ok(app: &TestApp, path: &str, expected_content_type: &str, file_path: &str) {
    let resp = app.get(path).await;
    assert!(resp.status().is_success(), "{} not served", path);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains(expected_content_type),
        "{} has wrong content-type: {}",
        path,
        ct
    );
    let body = resp.text().await.expect("Failed to read asset body");
    let expected = fs::read_to_string(file_path).expect("Failed to read asset file");
    assert_eq!(body, expected, "{} does not match file content", path);
}

#[tokio::test]
async fn assets_are_served() {
    let app = spawn_app().await;

    // Check HTML index page
    let index_resp = app.get("/").await;
    assert!(index_resp.status().is_success(), "Index page not served");
    let index_body = index_resp.text().await.expect("Failed to read index body");
    assert!(
        index_body.contains("<title>URL Shortener | Home</title>"),
        "Index page missing expected <title>"
    );
    assert!(
        index_body.contains("/static/landing.css"),
        "Index page missing CSS link"
    );
    assert!(
        index_body.contains("/static/scripts.js"),
        "Index page missing JS script"
    );

    // Check CSS asset
    assert_asset_ok(&app, "/static/landing.css", "css", "static/landing.css").await;

    // Check JS asset
    assert_asset_ok(
        &app,
        "/static/scripts.js",
        "javascript",
        "static/scripts.js",
    )
    .await;
}
