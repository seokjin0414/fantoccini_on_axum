use anyhow::{Result, anyhow};
use std::str::FromStr;
use std::{env::var, net::SocketAddr};

use crate::handlers::{
    legacy_kepco::{
        kepco::get_3year_kepco_data_of_handler,
        pp_kepco::{get_latest_3_pp_paid_data_handler, get_pp_all_periods_paid_data_handler},
    },
    pp::user_info::get_user_info_handler,
};
use axum::extract::DefaultBodyLimit;
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
        .route(
            "/crawling/legacy_kepco/3year",
            post(get_3year_kepco_data_of_handler),
        )
        .route(
            "/crawling/pp/paid/all-periods",
            post(get_pp_all_periods_paid_data_handler),
        )
        .route(
            "/crawling/pp/paid/latest-3-data",
            post(get_latest_3_pp_paid_data_handler),
        )
        .route("/crawling/pp/user-info", post(get_user_info_handler));

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
