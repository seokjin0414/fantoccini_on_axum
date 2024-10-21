use anyhow::{anyhow, Result};
use std::str::FromStr;
use std::{env::var, net::SocketAddr};

use crate::handlers::legacy_kepco_handlers::{
    kepco::get_3year_kepco_data_of_handler,
    pp_kepco::{
        get_latest_3_pp_paid_data_handler, get_pp_all_periods_paid_data_handler
    },
};
use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderName};
use axum::routing::post;
use chrono::{DateTime, Utc};

#[inline]
pub async fn server_initializer(
    start: tokio::time::Instant,
    server_start_time: DateTime<Utc>,
) -> Result<String> {
    // 각종 환경변수들을 여기서 가져올 것.
    // Save env. variables here.
    let app_name_version: String = match var("APP_NAME_VERSION") {
        Ok(var) => var,
        Err(e) => {
            return Err(anyhow!("Could not find var APP_NAME_VERSION: {:?}", e));
        }
    };
    let db_conn_url: String = match var("DB_CONN_URL") {
        Ok(var) => var,
        Err(e) => {
            return Err(anyhow!("Could not find var DB_CONN_URL: {:?}", e));
        }
    };
    let smtp_username: String = match var("SMTP_USERNAME") {
        Ok(var) => var,
        Err(e) => {
            return Err(anyhow!("Could not find var SMTP_USERNAME: {:?}", e));
        }
    };
    let smtp_password: String = match var("SMTP_PASSWORD") {
        Ok(var) => var,
        Err(e) => {
            return Err(anyhow!("Could not find var SMTP_PASSWORD: {:?}", e));
        }
    };
    let timezone: String = match var("TIMEZONE") {
        Ok(var) => var,
        Err(e) => return Err(anyhow!("Could not find var TIMEZONE: {:?}", e)),
    };
    let bucket_name: String = match var("BUCKET_NAME") {
        Ok(var) => var,
        Err(e) => return Err(anyhow!("Could not find var BUCKET_NAME: {:?}", e)),
    };

    // 호스팅할 IP를 여기서 %str로 파싱하거나 환경변수에서 로딩할 것.
    // Load host IP address here.
    let hosting_address: SocketAddr = match SocketAddr::from_str("[::]:30737") {
        Ok(addr) => addr,
        Err(e) => {
            return Err(anyhow!("Could not parse socket address: {:?}", e));
        }
    };

    // 인증 필요 없는 자료용. x-api-key로 대부분 접근은 걸러져서 민감하지 않은 정보 표출할 때 사용. 또는 테스트용.
    // For insensitive information that only requires x-api-key filtering. Or for testing.
    let insensitives_router: axum::Router = axum::Router::new()
        .route("/crawling/legacy_kepco_models/3year", post(get_3year_kepco_data_of_handler))
        .route("/crawling/power_planner_models/paid/all-periods", post(get_pp_all_periods_paid_data_handler))
        .route("/crawling/power_planner_models/paid/latest-3-data", post(get_latest_3_pp_paid_data_handler))
        ;

    // 최종 라우터.
    // The final router.
    let app: axum::Router = axum::Router::new()
        .merge(insensitives_router)
        .layer(DefaultBodyLimit::disable()); // 64MB

    // Tokio TCP listener에 IP를 연결해주고 오류처리.
    // Bind IP address to the Tokio TCP listener here.
    let listener: tokio::net::TcpListener =
        match tokio::net::TcpListener::bind(hosting_address).await {
            Ok(listener) => listener,
            Err(e) => {
                return Err(anyhow!("Could not initialize TcpListener: {:?}", e));
            }
        };

    // 나중에 오류처리로 넘길 것.
    // Handle error later.
    println!(
        "{}",
        format!(
            "{} started successfully on {} in {:?}.",
            app_name_version,
            hosting_address,
            start.elapsed()
        )
    );

    // 여기서 앱을 Axum으로 서빙.
    // Serve app with Axum here.
    match axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Axum could not serve app: {:?}", e));
        }
    };

    Ok(String::from("Server exiting."))
}
