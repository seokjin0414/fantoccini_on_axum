use anyhow::anyhow;
use axum::{
    response::IntoResponse,
    Json,
};

use crate::{
    models::{
        handler::pp::commons::{
            HOME_URL, PpRequestBody,
        },
        error::response_errors_def::ErrorResponseCode,
        driver::chromes::LOCAL_URL,
    },
    utils::drivers::{
        create_client,
        go_to_url,
    },
};

pub async fn get_pp_all_periods_paid_data_handler(
    Json(params): Json<PpRequestBody>,
) -> Result<impl IntoResponse, ErrorResponseCode >{

    let client = create_client(LOCAL_URL, PpRequestBody::test_state(&params))
        .await
        .map_err(|e| {
            ErrorResponseCode::CREATE_CLIENT
        })?;

    go_to_url(&client.clone(), HOME_URL).await.map_err(|e| {
        ErrorResponseCode::CREATE_CLIENT
    })?;



    client.delete_all_cookies().await.map_err(|e| {
        eprintln!("Failed to delete client cookies: {:?}", e);
        ErrorResponseCode::CREATE_CLIENT
    })?;

    Ok(())
}