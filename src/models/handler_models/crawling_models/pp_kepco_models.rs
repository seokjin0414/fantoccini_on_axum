use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde_derive::{Deserialize, Serialize};

// request body
#[derive(Deserialize)]
pub struct PpKepcoRequestBody {
    pub userId: String,
    pub userPw: String,
    pub userNum: String,
}

// 파워 플레너 요금 데이터
#[derive(Serialize, Debug)]
pub struct PpKepcoData {
    pub claim_date: NaiveDate,
    pub usage: f64,
    pub paid: i64,
}

#[derive(Serialize, Debug)]
pub struct MetaResponseData {}

#[derive(Serialize, Debug)]
pub struct PpKepcoPaidDataResponse {
    pub data: Vec<PpKepcoData>,
    pub meta: MetaResponseData,
}

impl IntoResponse for PpKepcoPaidDataResponse {
    fn into_response(self) -> axum::response::Response {
        let body = Json(self);
        (StatusCode::OK, body).into_response()
    }
}
