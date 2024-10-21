use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde_derive::{Deserialize, Serialize};

// 파워 플레너 요금 데이터
#[derive(Serialize, Debug)]
pub struct PpAllPeriodsPaidData {
    pub claim_date: NaiveDate,
    pub usage: f64,
    pub paid: i64,
}

#[derive(Serialize, Debug)]
pub struct MetaResponseData {}

#[derive(Serialize, Debug)]
pub struct PpAllPeriodsPaidDataResponse {
    pub data: Vec<PpAllPeriodsPaidData>,
    pub meta: MetaResponseData,
}

impl IntoResponse for PpAllPeriodsPaidData {
    fn into_response(self) -> axum::response::Response {
        let body = Json(self);
        (StatusCode::OK, body).into_response()
    }
}
