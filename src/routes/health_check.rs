//! # Health Check Handler
//!
//! This module provides the health check endpoint for monitoring the service status.
//! The health check endpoint is used by load balancers, monitoring systems, and
//! other services to verify that the URL shortener service is running and healthy.

use crate::response::ApiResponse;

/// Health check endpoint handler.
///
/// This handler provides a simple health check endpoint that returns a success
/// response if the service is running. It's commonly used by load balancers,
/// monitoring systems, and orchestration platforms to verify service health.
///
/// # Endpoint
///
/// `GET /api/health_check`
///
/// # Response
///
/// Returns a JSON response with the following structure:
///
/// ```json
/// {
///   "success": true,
///   "message": "ok",
///   "status": 200,
///   "time": "2025-01-18T12:00:00Z",
///   "data": null
/// }
/// ```
///
/// # Status Codes
///
/// - `200 OK` - Service is healthy and running
///
/// # Tracing
///
/// This handler is instrumented with tracing for request monitoring and debugging.
/// The span name is "health check" for easy identification in logs.
///
/// # Examples
///
/// ```bash
/// # Check service health
/// curl http://localhost:8000/api/health_check
///
/// # Expected response
/// {
///   "success": true,
///   "message": "ok",
///   "status": 200,
///   "time": "2025-01-18T12:00:00Z",
///   "data": null
/// }
/// ```
///
/// # Usage in Monitoring
///
/// This endpoint can be used with monitoring tools like:
/// - Prometheus health checks
/// - Kubernetes liveness/readiness probes
/// - Load balancer health checks
/// - Application monitoring dashboards
#[tracing::instrument(name = "health check")]
pub async fn health_check() -> ApiResponse<()> {
    ApiResponse::success(())
}
