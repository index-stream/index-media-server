use crate::api::router::{HttpRequest, HttpResponse};
use serde_json;
use std::future::Future;
use std::pin::Pin;

/// Handle ping endpoint
pub fn handle_ping(_request: &HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    Box::pin(async move {
        let response_body = serde_json::json!({
            "status": "ok",
            "message": "Index Media Server is running",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        Ok(HttpResponse::new(200)
            .with_cors()
            .with_json_body(&response_body.to_string()))
    })
}