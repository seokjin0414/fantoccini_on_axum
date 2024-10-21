use super::errors_def::ErrorResponseCode;
use axum::http::StatusCode;

impl ErrorResponseCode {
    pub const CONNECTION_POOL_ERROR: ErrorResponseCode = ErrorResponseCode {
        code: 4000,
        message:
            "Failed to get a database client from the connection pool; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const TRANSACTION_ERROR: ErrorResponseCode = ErrorResponseCode {
        code: 4001,
        message: "Failed to start a database transaction.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const COMMIT_ERROR: ErrorResponseCode = ErrorResponseCode {
        code: 4002,
        message: "Transaction commit failed.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const INCORRECT_CREDENTIALS: ErrorResponseCode = ErrorResponseCode {
        code: 4003,
        message: "The provided credentials are incorrect.",
        status_code: StatusCode::UNAUTHORIZED,
    };
    pub const SQL_QUERY_FAILED: ErrorResponseCode = ErrorResponseCode {
        code: 4004,
        message: "SQL query execution failed; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const AFFECTED_ROWS_ZERO: ErrorResponseCode = ErrorResponseCode {
        code: 4005,
        message: "SQL query went through, but affected rows is zero; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const COULD_NOT_DELETE_FILE_S3: ErrorResponseCode = ErrorResponseCode {
        code: 4006,
        message: "Could not delete file from S3; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const LINK_EXPIRED: ErrorResponseCode = ErrorResponseCode {
        code: 4007,
        message: "The provided link has expired.",
        status_code: StatusCode::GONE,
    };
    pub const LINK_INVALID: ErrorResponseCode = ErrorResponseCode {
        code: 4008,
        message: "The provided link is invalid.",
        status_code: StatusCode::BAD_REQUEST,
    };
    pub const CACHE_INCOHERENCE: ErrorResponseCode = ErrorResponseCode {
        code: 4009,
        message: "Cache incoherence detected; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const NO_CONTENT: ErrorResponseCode = ErrorResponseCode {
        code: 4010,
        message: "No content received.",
        status_code: StatusCode::BAD_REQUEST,
    };
    pub const JOIN_ERROR: ErrorResponseCode = ErrorResponseCode {
        code: 4011,
        message: "Join error occurred; please contact server admin.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const NOT_LOGGED_IN: ErrorResponseCode = ErrorResponseCode {
        code: 4012,
        message: "User is not logged in.",
        status_code: StatusCode::UNAUTHORIZED,
    };
    pub const COULD_NOT_DECODE_JWT: ErrorResponseCode = ErrorResponseCode {
        code: 4013,
        message: "Could not decode JWT.",
        status_code: StatusCode::BAD_REQUEST,
    };
    pub const NO_AUTHORITY: ErrorResponseCode = ErrorResponseCode {
        code: 4014,
        message: "User does not have the necessary authority.",
        status_code: StatusCode::FORBIDDEN,
    };

    pub const QUERY_PREPARATION_FAILED: ErrorResponseCode = ErrorResponseCode {
        code: 4015,
        message: "Failed to prepare statement.",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const FALLBACK_ERROR: ErrorResponseCode = ErrorResponseCode {
        code: 4016,
        message: "Path unavailable!",
        status_code: StatusCode::NOT_FOUND,
    };
}
