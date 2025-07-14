use std::time::Duration;
use crate::{
    handlers::pp::commons::pp_login,
    models::{
        driver::chromes::LOCAL_URL,
        error::response_errors_def::ErrorResponseCode,
        handler::pp::{
            commons::{PpRequestBody, USER_INFO_URL, USER_SELECT_CHARGE_URL},
            user_info::{
                CONTRACT_TYPE, CONTRACT_TYPE_ID, PURPOSE, PURPOSE_ID, UserInfo, contract_vec,
                purpose_vec,
            },
        },
        response::commons::basic_response,
    },
    utils::drivers::*,
};
use anyhow::{Result, anyhow};
use axum::{Json, response::IntoResponse};
use fantoccini::{Client, Locator};
use regex::Regex;

pub async fn get_user_info_handler(
    Json(params): Json<PpRequestBody>,
) -> Result<impl IntoResponse, ErrorResponseCode> {
    let start = std::time::Instant::now();

    let client = create_client(LOCAL_URL, PpRequestBody::test_state(&params))
        .await
        .map_err(|_| ErrorResponseCode::CREATE_CLIENT)?;

    pp_login(&client, params)
        .await
        .map_err(|_| ErrorResponseCode::PP_LOGIN)?;

    let user_info = pp_user_info(&client)
        .await
        .map_err(|_| ErrorResponseCode::PP_USER_INFO)?;

    clean_client(&client)
        .await
        .map_err(|_| ErrorResponseCode::CLEAN_CLIENT)?;

    Ok(basic_response(user_info, start.elapsed()))
}

async fn pp_user_info(client: &Client) -> Result<UserInfo> {
    go_to_url(client, USER_INFO_URL).await?;
    wait_element(&client, Locator::Css("#table2")).await?;

    let user_number = text_element(
        &client,
        Locator::Css(
            "#contents > div.table_info > table > tbody > tr:nth-child(1) > td:nth-child(2)",
        ),
    )
    .await?;

    let contract = text_element(
        &client,
        Locator::Css(
            "#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(2)",
        ),
    )
    .await?;

    let contract_power = text_element(
        &client,
        Locator::Css(
            "#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(4)",
        ),
    )
    .await?;

    let inspection_day = text_element(
        &client,
        Locator::Css("#table2 > tbody > tr:nth-child(1) > td:nth-child(4)"),
    )
    .await?;

    click_element(&client, Locator::Css("#tab3 > a")).await?;

    let instrument_number = text_element(
        &client,
        Locator::Css("#table3 > tbody > tr:nth-child(1) > td:nth-child(2)"),
    )
    .await?;

    println!("pp_user_info successfully");
    Ok(UserInfo {
        user_number,
        contract_type_id: extract_id(&contract, CONTRACT_TYPE)?,
        purpose_id: pp_user_select_charge_info(&client, &contract).await?,
        contract_power: parse_day(&contract_power)? as f64,
        inspection_day: parse_day(&inspection_day)?,
        instrument_number,
    })
}

async fn pp_user_select_charge_info(client: &Client, contract: &str) -> Result<i16> {
    go_to_url(client, USER_SELECT_CHARGE_URL).await?;

    wait_for_element_display_none(
        client,
        Locator::Id("backgroundLayer"),
        Duration::from_secs(15),
    )
        .await?;

    let pricing_plan = text_element(&client, Locator::Id("spanCNTR_KND_NM")).await?;
    println!("pricing_plan: {}", pricing_plan);
    println!("contract: {}", contract);

    let re = Regex::new(r"(고압.|저압.)")?;
    let target = re.replace_all(contract, "").to_string();
    println!("target: {}", target);

    let result = pricing_plan.replace(&target, "").replace(" ", "");
    println!("result: {}", result);

    println!("pp_user_select_charge_info successfully");
    Ok(extract_id(&result, PURPOSE)?)
}

fn parse_day(input: &str) -> Result<i16> {
    let re = Regex::new(r"\d+").map_err(|e| anyhow!("Invalid regex: {:?}", e))?;

    if let Some(mat) = re.find(input) {
        let day_str = mat.as_str();
        let day = day_str
            .parse::<i16>()
            .map_err(|e| anyhow!("Failed to parse day '{}': {:?}", day_str, e))?;
        return Ok(day);
    }

    Ok(0)
}

fn extract_id(input: &str, kind: &str) -> Result<i16> {
    let (set, default) = match kind {
        CONTRACT_TYPE => (contract_vec(), CONTRACT_TYPE_ID),
        PURPOSE => (purpose_vec(), PURPOSE_ID),
        _ => return Err(anyhow!("Failed to extract_id: {:?} ({:?}) ", kind, input)),
    };

    for (pattern, id) in set {
        if input == pattern {
            return Ok(id);
        }
    }

    Ok(default)
}