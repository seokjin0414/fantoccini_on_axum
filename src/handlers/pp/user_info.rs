use anyhow::{anyhow, Result};
use axum::{
    response::IntoResponse,
    Json,
};
use fantoccini::{Client, Locator};
use regex::Regex;
use crate::{
    models::{
        handler::pp::{
            commons::{
                USER_INFO_URL, PpRequestBody,
            },
            user_info::UserInfo,
        },
        error::response_errors_def::ErrorResponseCode,
        driver::chromes::LOCAL_URL,
        response::commons::basic_response,
    },
    utils::drivers::*,
    handlers::pp::commons::pp_login,
};

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

async fn pp_user_info(client: &Client) -> anyhow::Result<UserInfo> {
    go_to_url(client, USER_INFO_URL).await?;
    wait_element(&client, Locator::Css("#table2")).await?;

    let user_number = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(1) > td:nth-child(2)"),
    ).await?;

    let contract_type = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(2)"),
    ).await?;

    let mut contract_power = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(4)"),
    ).await?;

    let inspection_day = text_element(
        &client,
        Locator::Css("#table2 > tbody > tr:nth-child(1) > td:nth-child(4)"),
    ).await?;

    click_element(&client, Locator::Css("#tab3 > a")).await?;

    let instrument_number = text_element(
        &client,
        Locator::Css("#table3 > tbody > tr:nth-child(1) > td:nth-child(2)"),
    ).await?;

    Ok(UserInfo {
        user_number,
        contract_type,
        contract_power: contract_power.replace("kw", ""),
        inspection_day: parse_day(&inspection_day)?,
        instrument_number,
    })
}

fn parse_day(input: &str) -> Result<i16> {
    let re = Regex::new(r"\d+").map_err(|e| anyhow!("Invalid regex: {:?}", e))?;

    if let Some(mat) = re.find(input) {
        let day_str = mat.as_str();
        let day = day_str.parse::<i16>()
            .map_err(|e| anyhow!("Failed to parse day '{}': {:?}", day_str, e))?;
        return Ok(day);
    }

    Ok(0)
}
