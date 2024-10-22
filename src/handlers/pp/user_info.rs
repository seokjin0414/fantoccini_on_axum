use axum::{
    response::IntoResponse,
    Json,
};

use crate::models::{
    handler::pp::commons::PpRequestBody,
    error::response_errors_def::ErrorResponseCode,
};

pub async fn get_pp_all_periods_paid_data_handler(
    Json(params): Json<PpRequestBody>,
) -> Result<impl IntoResponse, ErrorResponseCode >{




}