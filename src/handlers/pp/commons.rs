use std::time::Duration;
use anyhow::{anyhow, Result};
use fantoccini::{Client, ClientBuilder, Locator};

use crate::utils::drivers::*;
use crate::models::{
    driver::chromes::ChromeOptions,
    handler::pp::{
        commons::{
            PP_URL, HOME_URL, USER_INFO_URL,
            PpRequestBody,
        },
        user_info::UserInfo,
    },
};

pub async fn pp_login(client: &Client, params: PpRequestBody) -> Result<()> {
    // 페이지 이동
    go_to_url(client, PP_URL).await?;

    // 공지 팝업 로드 대기
    wait_element(client,  Locator::Id("notice_auto_popup")).await?;
    //공지 팝업 비활성화
    click_element(client, Locator::XPath("/html/body/div[2]/div[3]/label")).await?;

    // id 입력 로드 대기
    wait_element(client, Locator::Id("RSA_USER_ID")).await?;
    // id 입력
    enter_value_in_element(client, Locator::Id("RSA_USER_ID"), &params.userId).await?;
    // pw 입력
    enter_value_in_element(client, Locator::Id("RSA_USER_PWD"), &params.userPw).await?;
    // 로그인 버튼 클릭
    click_element(client, Locator::Css("#intro_form > form > fieldset > input.intro_btn")).await?;

    // // 로딩 대기
    wait_for_element_display_none(client, Locator::Id("backgroundLayer"), Duration::from_secs(10)).await?;

    // user_num selector 클릭
    click_element(client, Locator::XPath("/html/body/div[1]/div[1]/div/div/a[2]")).await?;

    // user_num 클릭
    click_element(
        client,
        Locator::XPath(
            format!(
                "/html/body/div[1]/div[1]/div/div/ul/li[1]/a[text()='{}']",
                params.userNum,
            ).as_str(),
        )
    ).await?;

    // // 로딩 대기
    wait_for_element_display_none(client, Locator::Id("backgroundLayer"), Duration::from_secs(10)).await?;

    println!("pp_login successfully");
    Ok(())
}

pub async fn pp_user_info(client: &Client) -> Result<UserInfo> {
    // 페이지 이동
    go_to_url(client, USER_INFO_URL).await?;

    wait_element(&client, Locator::Css("#table2")).await?;

    let user_number = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(1) > td:nth-child(2)"),
    ).await?;
    println!("user_number successfully");

    let contract_type = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(2)"),
    ).await?;
    println!("contract_type successfully");

    let contract_power = text_element(
        &client,
        Locator::Css("#contents > div.table_info > table > tbody > tr:nth-child(2) > td:nth-child(4)"),
    ).await?;
    println!("contract_power successfully");

    let inspection_day = text_element(
        &client,
        Locator::Css("#table2 > tbody > tr:nth-child(1) > td:nth-child(4)"),
    ).await?;
    println!("inspection_day successfully");

    let instrument_number = text_element(
        &client,
        Locator::Css("#table3 > tbody > tr:nth-child(1) > td:nth-child(2)"),
    ).await?;
    println!("instrument_number successfully");


    Ok(UserInfo {
        user_number,
        contract_type,
        contract_power,
        inspection_day,
        instrument_number,
    })
}