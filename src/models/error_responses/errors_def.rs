use std::collections::BTreeMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::json;

pub const ERRORS_SOURCE_CODE: &str = include_str!("errors.rs");

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

// Use where additional metadata is required in a separate field.
pub struct ErrorResponseCodeOwnedStrWithMeta {
    pub code: u16,
    pub message: String,
    pub status_code: StatusCode,
    pub meta: Vec<ErrorResponseMeta>,
}

// Use where additional metadata is required with a key-value structure.
pub struct ErrorResponseCodeOwnedStrWithKVMeta {
    pub code: u16,
    pub message: String,
    pub status_code: StatusCode,
    pub meta: BTreeMap<String, ErrorResponseMeta>,
}

pub enum ErrorResponseMeta {
    String(String),
    Integer(i128),
    TimestampTz(DateTime<Utc>),
    // Float(f64),
}

impl serde::Serialize for ErrorResponseMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ErrorResponseMeta::String(value) => serializer.serialize_str(value),
            ErrorResponseMeta::Integer(value) => serializer.serialize_i128(*value),
            ErrorResponseMeta::TimestampTz(value) => serializer.serialize_str(&value.to_rfc3339()),
            // ErrorResponseMeta::Float(value) => serializer.serialize_f64(*value),
        }
    }
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

impl IntoResponse for ErrorResponseCodeOwnedStr {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "code": self.code,
            "message": self.message,
            "status": self.status_code.as_u16()
        });
        (self.status_code, Json(body)).into_response()
    }
}

impl IntoResponse for ErrorResponseCodeOwnedStrWithMeta {
    fn into_response(self) -> Response {
        let body = json!({
            "code": self.code,
            "message": self.message,
            "status": self.status_code.as_u16(),
            "meta": self.meta
        });
        (self.status_code, Json(body)).into_response()
    }
}

impl IntoResponse for ErrorResponseCodeOwnedStrWithKVMeta {
    fn into_response(self) -> Response {
        let body = json!({
            "code": self.code,
            "message": self.message,
            "status": self.status_code.as_u16(),
            "meta": self.meta
        });
        (self.status_code, Json(body)).into_response()
    }
}

impl ErrorResponseCode {
    pub fn with_message(self, message: String) -> ErrorResponseCodeOwnedStr {
        ErrorResponseCodeOwnedStr {
            code: self.code,
            message: format!("{} {}", self.message, message),
            status_code: self.status_code,
        }
    }

    pub fn with_metadata(
        self,
        message: Vec<ErrorResponseMeta>,
    ) -> ErrorResponseCodeOwnedStrWithMeta {
        ErrorResponseCodeOwnedStrWithMeta {
            code: self.code,
            message: self.message.to_owned(),
            status_code: self.status_code,
            meta: message,
        }
    }

    pub fn with_kv_metadata(
        self,
        message: BTreeMap<String, ErrorResponseMeta>,
    ) -> ErrorResponseCodeOwnedStrWithKVMeta {
        ErrorResponseCodeOwnedStrWithKVMeta {
            code: self.code,
            message: self.message.to_owned(),
            status_code: self.status_code,
            meta: message,
        }
    }
}
