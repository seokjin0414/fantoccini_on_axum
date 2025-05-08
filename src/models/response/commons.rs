use axum::{http::StatusCode, response::IntoResponse, response::Response, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponseStruct<D, M>
where
    D: Serialize,
    M: Serialize,
{
    pub data: D,
    pub meta: M,
}

impl<D, M> IntoResponse for GenericResponseStruct<D, M>
where
    D: Serialize,
    M: Serialize,
{
    fn into_response(self) -> Response {
        let body = Json(self);
        (StatusCode::OK, body).into_response()
    }
}

#[derive(Serialize, Debug)]
pub struct TimeMeta {
    pub time_taken: String,
    pub times_tamp: DateTime<Utc>,
}

pub fn basic_response<D: Serialize>(data: D, run_time: tokio::time::Duration) -> Response {
    GenericResponseStruct {
        data,
        meta: TimeMeta {
            time_taken: format!("{:?}", run_time),
            times_tamp: Utc::now(),
        },
    }
    .into_response()
}
