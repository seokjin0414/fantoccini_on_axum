use anyhow::Result;
use fantoccini::{Client, Locator};
use std::time::Duration;

use crate::models::handler::pp::commons::{PP_URL, PpRequestBody};
use crate::utils::drivers::*;

pub async fn pp_login(client: &Client, params: PpRequestBody) -> Result<()> {
    go_to_url(client, PP_URL).await?;

    wait_element(client, Locator::Id("notice_auto_popup")).await?;
    //공지 팝업 비활성화
    let _ = click_element(client, Locator::XPath("/html/body/div[2]/div[3]/label")).await;

    wait_element(client, Locator::Id("RSA_USER_ID")).await?;
    enter_value_in_element(client, Locator::Id("RSA_USER_ID"), &params.userId).await?;
    enter_value_in_element(client, Locator::Id("RSA_USER_PWD"), &params.userPw).await?;
    click_element(
        client,
        Locator::Css("#intro_form > form > fieldset > input.intro_btn"),
    )
    .await?;

    wait_for_element_display_none(
        client,
        Locator::Id("backgroundLayer"),
        Duration::from_secs(10),
    )
    .await?;

    let user_num_css = format!("ul > li > a[href='#{}']", params.userNum);
    let selector_json = serde_json::to_string(&user_num_css).unwrap_or_default();
    let script = format!(
        "var a = document.querySelector({}); if (a) {{ a.click(); }}",
        selector_json
    );
    
    // user_num 클릭
    script_execute(client, &script).await?;
    
    wait_for_element_display_none(
        client,
        Locator::Id("backgroundLayer"),
        Duration::from_secs(10),
    )
    .await?;

    println!("pp_login successfully");
    Ok(())
}