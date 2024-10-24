use anyhow::anyhow;
use axum::{
    response::IntoResponse,
    Json,
};

use crate::{
    models::{
        handler::pp::commons::{
            PP_URL, HOME_URL, PpRequestBody,
        },
        error::response_errors_def::ErrorResponseCode,
        driver::chromes::LOCAL_URL,
        response::commons::basic_response,
    },
    utils::drivers::{
        create_client,
        go_to_url,
    },
    handlers::pp::commons::pp_login,
};
use crate::handlers::pp::commons::pp_user_info;

pub async fn get_user_info_handler(
    Json(params): Json<PpRequestBody>,
) -> Result<impl IntoResponse, ErrorResponseCode >{
    let start = std::time::Instant::now();

    let client = create_client(LOCAL_URL, PpRequestBody::test_state(&params))
        .await
        .map_err(|e| {
            ErrorResponseCode::CREATE_CLIENT
        })?;

    pp_login(&client, params).await.map_err(|e| {
        ErrorResponseCode::PP_LOGIN
    })?;

    let user_info = pp_user_info(&client).await.map_err(|e| {
        ErrorResponseCode::PP_USER_INFO
    })?;

    println!("@@@@@@@@@@@@@@@@@@@@@@@@@ TEST @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
    client.delete_all_cookies().await.map_err(|e| {
        eprintln!("Failed to delete client cookies: {:?}", e);
        ErrorResponseCode::CREATE_CLIENT
    })?;

    Ok(basic_response(user_info, start.elapsed()))
}


