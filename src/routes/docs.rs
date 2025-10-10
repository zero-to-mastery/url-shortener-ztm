//! # API Documentation Routes
//!
//! This module provides routes for serving API documentation including
//! the OpenAPI specification and Swagger UI interface.

use axum::response::Html;
use axum_macros::debug_handler;

/// Serve the OpenAPI specification as YAML.
///
/// This handler serves the OpenAPI 3.0 specification file that describes
/// all the API endpoints, request/response formats, and authentication
/// requirements for the URL shortener service.
///
/// # Endpoint
///
/// `GET /api/docs/openapi.yaml`
///
/// # Response
///
/// Returns the OpenAPI specification in YAML format with appropriate
/// Content-Type header for YAML files.
///
/// # Content Type
///
/// `application/yaml`
///
#[debug_handler]
pub async fn serve_openapi_spec() -> impl axum::response::IntoResponse {
    let yaml_content = include_str!("../../openapi.yaml");

    axum::response::Response::builder()
        .header("content-type", "application/yaml")
        .body(axum::body::Body::from(yaml_content))
        .unwrap()
}

/// Serve the Swagger UI interface.
///
/// This handler serves an HTML page containing the Swagger UI interface
/// for interactive API documentation. The Swagger UI loads the OpenAPI
/// specification from the `/api/docs/openapi.yaml` endpoint.
///
/// # Endpoint
///
/// `GET /api/docs`
///
/// # Response
///
/// Returns an HTML page with embedded Swagger UI that provides:
/// - Interactive API documentation
/// - Request/response examples
/// - Ability to test API endpoints directly from the browser
/// - Authentication support for protected endpoints
///
/// # Features
///
/// - **Interactive Testing**: Try out API endpoints directly from the UI
/// - **Request/Response Examples**: View sample requests and responses
/// - **Authentication**: Support for API key authentication
/// - **Schema Validation**: Automatic validation of request/response formats
///
#[debug_handler]
pub async fn serve_swagger_ui() -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="URL Shortener API Documentation" />
    <title>URL Shortener API - Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.10.3/swagger-ui.css" />
    <style>
        body {
            margin: 0;
            padding: 0;
        }
        #swagger-ui {
            width: 100%;
            height: 100vh;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.10.3/swagger-ui-bundle.js" crossorigin></script>
    <script>
        window.onload = () => {
            window.ui = SwaggerUIBundle({
                url: '/api/docs/openapi.yaml',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.presets.standalone
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                tryItOutEnabled: true,
                requestInterceptor: (req) => {
                    // Add any custom request interceptors here
                    return req;
                },
                responseInterceptor: (res) => {
                    // Add any custom response interceptors here
                    return res;
                }
            });
        };
    </script>
</body>
</html>"#;

    Html(html.to_string())
}
