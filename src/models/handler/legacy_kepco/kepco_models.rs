use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde_derive::{Serialize};

// 한전온 요금 데이터
#[derive(Serialize, Debug)]
pub struct KepcoData {
    pub claim_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub usage: f64,
    pub amount: i64,
    pub paid: i64,
    pub unpaid: i64,
    pub payment_method: Option<String>,
    pub payment_date: Option<NaiveDate>,
}

#[derive(Serialize, Debug)]
pub struct MetaResponseData {}

#[derive(Serialize, Debug)]
pub struct ThreeYearKepcoDataResponse {
    pub data: Vec<KepcoData>,
    pub meta: MetaResponseData,
}

impl IntoResponse for ThreeYearKepcoDataResponse {
    fn into_response(self) -> axum::response::Response {
        let body = Json(self);
        (StatusCode::OK, body).into_response()
    }
}
