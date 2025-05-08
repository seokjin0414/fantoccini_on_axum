use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;


#[derive(Clone)]
pub struct ErrorResponseCode {
    pub code: u16,
    pub message: &'static str,
    pub status_code: StatusCode,
}

// Use for dynamic error messages.
pub struct ErrorResponseCodeOwnedStr {
    pub code: u16,
    pub message: String,
    pub status_code: StatusCode,
}

impl IntoResponse for ErrorResponseCode {
    fn into_response(self) -> Response {
        let body = json!({
            "code": self.code,
            "message": self.message,
            "status": self.status_code.as_u16()
        });
        (self.status_code, Json(body)).into_response()
    }
}