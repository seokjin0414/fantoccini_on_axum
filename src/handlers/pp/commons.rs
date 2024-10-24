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