use super::response_errors_def::ErrorResponseCode;
use axum::http::StatusCode;

impl ErrorResponseCode {
    pub const CREATE_CLIENT: ErrorResponseCode = ErrorResponseCode {
        code: 5001,
        message: "Could not create_client!",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const PP_LOGIN: ErrorResponseCode = ErrorResponseCode {
        code: 5002,
        message: "Could not pp_login!",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const PP_USER_INFO: ErrorResponseCode = ErrorResponseCode {
        code: 5003,
        message: "Could not pp_user_info!",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
    pub const CLEAN_CLIENT: ErrorResponseCode = ErrorResponseCode {
        code: 5003,
        message: "Could not clean_client!",
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };
}
