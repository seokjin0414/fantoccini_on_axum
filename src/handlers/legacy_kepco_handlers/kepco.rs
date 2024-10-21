use anyhow::{Context, Result};
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::{Datelike, NaiveDate};
use dashmap::DashMap;
use fantoccini::error::CmdError;
use fantoccini::{elements::Element, Client, ClientBuilder, Locator};
use serde_json::{json, Map, Value};
use std::{
    process::{Child, Command},
    sync::Arc,
};
use tokio::time::{timeout, Duration};

use crate::models::handler_models::legacy_kepco_models::kepco_models::{KepcoData, KepcoRequestBody};

// 한전 3년치 요금 조회 고객번호 기준
pub async fn get_3year_kepco_data_of_handler(
    Json(params): Json<KepcoRequestBody>,
) -> impl IntoResponse {
    let url = "http://localhost:4444";
    let user_id = &params.userId;
    let user_pw = &params.userPw;
    let user_number = &params.userNum;

    // driver path
    let chromedriver_path = "src/driver/chromedriver";
    let chrome_binary_path = "/usr/bin/google-chrome";

    // driver 실행
    let mut chromedriver_process = match Command::new(chromedriver_path)
        .arg("--port=4444")
        .arg(format!("--binary={}", chrome_binary_path))
        .spawn()
    {
        Ok(process) => process,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not chromedriver_process!: {:?}", e),
            )
                .into_response()
        }
    };

    // driver 대기
    tokio::time::sleep(Duration::from_secs(2)).await;

    // headless, disable-gpu option
    let capabilities: Map<String, Value> = match serde_json::from_value(json!({
        "goog:chromeOptions": {
            "binary": "/usr/bin/google-chrome",
            "args": ["--headless", "--disable-gpu"]
        }
    })) {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not acquire connection from pool!",
            )
                .into_response()
        }
    };

    let client = loop {
        match ClientBuilder::native()
            .capabilities(capabilities.clone())
            .connect(url)
            .await
        {
            Ok(client) => break client,
            Err(e) => {
                eprintln!("Retrying to connect to WebDriver: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };
    let client_arc = Arc::new(client.clone());

    // view size
    match client_arc.set_window_rect(0, 0, 774, 857).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set view size!: {:?}", e),
            )
                .into_response();
        }
    };
    // 페이지 이동
    match client_arc.goto("https://online.kepco.co.kr").await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to navigate goto!: {:?}", e),
            )
                .into_response();
        }
    };

    // menu button 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_btnSiteMap"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // menu button 클릭
    match click_element(&client_arc, Locator::Id("mf_wfm_header_gnb_btnSiteMap")).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // login form 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_mobileGoLogin"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to wait_for_element!",
            )
                .into_response()
        }
    };
    // login form 클릭
    match click_element(&client_arc, Locator::Id("mf_wfm_header_gnb_mobileGoLogin")).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // id 입력 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // id 입력
    match enter_value_in_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_id"),
        user_id,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to enter_value_in_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // pw 입력
    match enter_value_in_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_login_popup_wframe_ui_pw"),
        user_pw,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to enter_value_in_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // 로그인 버튼 클릭
    match click_element(
        &client_arc,
        Locator::Id("mf_wfm_header_gnb_login_popup_wframe_btn_login"),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // 요금 조회 버튼 클릭 반복 시도
    match click_element_with_retries(
        &client_arc,
        Locator::XPath("/html/body/div[2]/div[3]/div/div/div[4]/div/div[2]/div[1]/a[3]"),
        10,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // 필드 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_layout_inp_searchCustNo"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // 로딩 대기
    match wait_for_element_hidden(
        &client_arc,
        Locator::Id("mf_wq_uuid_1_wq_processMsgComp"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_hidden!: {:?}", e),
            )
                .into_response()
        }
    };

    // 사용자 번호 입력
    match enter_value_in_element(
        &client_arc,
        Locator::Id("mf_wfm_layout_inp_searchCustNo"),
        user_number,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to enter_value_in_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // 검색 버튼 클릭
    match click_element(&client_arc, Locator::Id("mf_wfm_layout_btn_search")).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // 상세 요금 버튼 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element!: {:?}", e),
            )
                .into_response()
        }
    };
    // 스크롤 강제 맨 아래
    match client_arc
        .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to scrollHeight!: {:?}", e),
            )
                .into_response()
        }
    };
    // 상세 요금 버튼 클릭
    match click_element(
        &client_arc,
        Locator::Id("mf_wfm_layout_ui_generator_0_btn_moveDetail"),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // '1년' 옵션을 선택
    match click_element_with_retries(&client_arc, Locator::XPath("//option[text()='1년']"), 10)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to click_element_with_retries!: {:?}", e),
            )
                .into_response()
        }
    };

    // 로딩 대기
    match wait_for_element_hidden(
        &client_arc,
        Locator::Id("mf_wq_uuid_1_wq_processMsgComp"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_hidden!: {:?}", e),
            )
                .into_response()
        }
    };

    // 자식 요소 ID, DashMap 에 저장
    let map = match get_children_ids_to_map(&client_arc, "mf_wfm_layout_ui_generator").await {
        Ok(map) => map,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not get_children_ids_to_map!: {:?}", e),
            )
                .into_response()
        }
    };

    // data from parent_id -> vec
    let mut data_vec = match parse_data_from_parent_ids(&client_arc, map).await {
        Ok(vec) => vec,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not parse_data_from_parent_ids!: {:?}", e),
            )
                .into_response()
        }
    };
    data_vec.sort_by(|a, b| b.claim_date.cmp(&a.claim_date));

    let reference_date = data_vec[data_vec.len() - 1]
        .claim_date
        .map(|date| format!("{}년 {:02}월", date.year(), date.month()))
        .unwrap_or_else(|| "N/A".to_string());

    // 뒤로 가기
    match client_arc.back().await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to navigate back!: {:?}", e),
            )
                .into_response()
        }
    };

    // select 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("mf_wfm_layout_slb_searchYm_input_0"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element!: {:?}", e),
            )
                .into_response()
        }
    };

    // 로딩 대기
    match wait_for_element_hidden(
        &client_arc,
        Locator::Id("mf_wq_uuid_1_wq_processMsgComp"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_hidden!: {:?}", e),
            )
                .into_response()
        }
    };

    // select 에서 reference_date 옵션의 인덱스 search
    let select_locator = Locator::Id("mf_wfm_layout_slb_searchYm_input_0");
    let mut option_index =
        match get_option_index(&client_arc, select_locator, &reference_date).await {
            Ok(u) => u,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to find option index!: {:?}", e),
                )
                    .into_response()
            }
        };

    option_index += 1;

    // 1year over data parsing
    let mut additional_data_vec = match parsing_options_data(
        &client_arc,
        select_locator,
        user_number,
        &option_index,
        &mut chromedriver_process,
    )
    .await
    {
        Ok(vec) => vec,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed parsing_options_data!: {:?}", e),
            )
                .into_response()
        }
    };

    // data 병합
    data_vec.append(&mut additional_data_vec);

    // 2분 동안 대기
    // println!("Waiting for 2 minutes...");
    // tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

    match client.delete_all_cookies().await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed delete_all_cookies!: {:?}", e),
            )
                .into_response()
        }
    }

    // ChromeDriver 프로세스 종료
    match chromedriver_process.kill() {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed chromedriver_process.kill!: {:?}", e),
            )
                .into_response()
        }
    };

    (StatusCode::OK, Json(data_vec)).into_response()
}

// 요소 대기
async fn wait_for_element(
    client: &Client,
    locator: Locator<'_>,
    chromedriver_process: &mut Child,
) -> Result<Option<Element>> {
    match client.wait().for_element(locator).await {
        Ok(element) => Ok(Some(element)),
        Err(e) => {
            eprintln!("Failed to find the element: {:?}\n {}", locator, e);
            client
                .clone()
                .close()
                .await
                .context("Failed to close client")?;
            chromedriver_process
                .kill()
                .expect("failed to kill ChromeDriver");
            Err(anyhow::anyhow!("Failed to find the element: {:?}", e))
        }
    }
}

// 요소 클릭
async fn click_element(client: &Client, locator: Locator<'_>) -> Result<()> {
    if let Ok(element) = client.find(locator).await {
        element
            .click()
            .await
            .context(format!("Failed to click the element: {:?}", locator))?;
        println!("Element clicked successfully: {:?}", locator);
    } else {
        eprintln!("Failed to find the element: {:?}", locator);
        return Err(anyhow::anyhow!("Failed to find the element: {:?}", locator));
    }
    Ok(())
}

// 요소에 값 입력
async fn enter_value_in_element(client: &Client, locator: Locator<'_>, text: &str) -> Result<()> {
    if let Ok(element) = client.find(locator).await {
        if let Err(e) = element.send_keys(text).await {
            eprintln!("Failed to enter text: {}", e);
        } else {
            println!("Text entered successfully: {:?}", locator);
        }
    } else {
        eprintln!("Failed to find the input element: {:?}", locator);
    }
    Ok(())
}

// 요소 비활성화 대기
async fn wait_for_element_hidden(
    client: &Client,
    locator: Locator<'_>,
    chromedriver_process: &mut Child,
    duration: Duration,
) -> Result<()> {
    let element = match wait_for_element(client, locator, chromedriver_process).await? {
        Some(element) => element,
        None => return Err(anyhow::anyhow!("Failed to find the element: {:?}", locator)),
    };

    let element_hidden = timeout(duration, async {
        loop {
            match element.attr("aria-hidden").await {
                Ok(Some(value)) if value == "true" => {
                    println!("Element is hidden (aria-hidden=\"true\")");
                    break;
                }
                Ok(_) => {
                    eprintln!("Element is not hidden, retrying...");
                }
                Err(e) => {
                    eprintln!("Failed to get aria-hidden attribute: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await;

    if element_hidden.is_err() {
        Err(anyhow::anyhow!(
            "Failed to find the element within the given duration"
        ))
    } else {
        Ok(())
    }
}

// 요소 클릭 반복
async fn click_element_with_retries(
    client: &Client,
    locator: Locator<'_>,
    max_attempts: u32,
) -> Result<()> {
    let mut attempts = 0;
    loop {
        if attempts >= max_attempts {
            return Err(anyhow::anyhow!(
                "Failed to click the element after {} attempts",
                max_attempts
            ));
        }
        match client.find(locator).await {
            Ok(element) => match element.click().await {
                Ok(_) => {
                    println!(
                        "Element clicked successfully after {} attempts",
                        attempts + 1
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!(
                        "Failed to click the element (attempt {}): {}",
                        attempts + 1,
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempts += 1;
                }
            },
            Err(e) => {
                eprintln!(
                    "Retrying to find the element (attempt {}): {}",
                    attempts + 1,
                    e
                );
                // 요소를 찾지 못하면 잠시 대기 후 다시 시도
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
    }
}

// select 요소에서 옵션 인덱스 찾기
async fn get_option_index(
    client: &Client,
    select_locator: Locator<'_>,
    text: &str,
) -> Result<usize> {
    let element = client
        .find(select_locator)
        .await
        .context("Failed to find select element")?;

    let options = element.find_all(Locator::XPath(".//option")).await?;
    for (index, option) in options.iter().enumerate() {
        if let Ok(option_text) = option.text().await {
            if option_text == text {
                return Ok(index);
            }
        }
    }
    Err(anyhow::anyhow!("Option with text '{}' not found", text))
}

// 자식 요소들의 ID -> DashMap
async fn get_children_ids_to_map(
    client: &Client,
    parent_id: &str,
) -> Result<Arc<DashMap<String, ()>>> {
    let script = format!(
        r#"
        let children = document.getElementById('{}').children;
        let ids = [];
        for (let i = 0; i < children.length; i++) {{
            ids.push(children[i].id);
        }}
        return ids;
        "#,
        parent_id
    );

    let result = client
        .execute(&script, vec![])
        .await
        .context("Failed to execute script to get children IDs")?;

    let ids: Vec<String> = result
        .as_array()
        .context("Expected an array from the script result")?
        .iter()
        .map(|v| {
            v.as_str()
                .context("Expected a string in the array")
                .map(|s| s.to_string())
        })
        .collect::<Result<Vec<String>>>()?;

    let map = Arc::new(DashMap::new());
    for id in ids {
        map.insert(id, ());
    }

    Ok(map)
}

// parsing 청구 기간
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let date_with_day = if date_str.len() == 7 {
        format!("{}.01", date_str) // 일자를 1로 설정
    } else {
        date_str.to_string()
    };

    NaiveDate::parse_from_str(&date_with_day, "%Y.%m.%d").context("Failed to parse date")
}

// parsing 대상 기간
fn parse_date_range(date_range: &str) -> Result<(Option<NaiveDate>, Option<NaiveDate>)> {
    let dates: Vec<&str> = date_range.split('-').collect();

    let start_date = parse_date(dates[0]).ok();
    let end_date = parse_date(dates[1]).ok();

    Ok((start_date, end_date))
}

// parsing 사용량
fn parse_use_kwh(kwh_str: &str) -> Result<f64> {
    let cleaned_str = kwh_str.replace(',', "").replace("kWh", "");
    cleaned_str
        .parse::<f64>()
        .context("Failed to parse use kWh")
}

// parsing 요금
fn parse_amount(amount_str: &str) -> Result<i64> {
    let amount_part = amount_str.split('원').next().unwrap_or(amount_str);

    let amount = amount_part.replace(',', "").replace('.', "");
    amount.parse::<i64>().context("Failed to parse amount")
}

// parsing 지불 방법, 기간
fn parse_payment_method(payment_str: &str) -> Result<(Option<String>, Option<NaiveDate>)> {
    let parts: Vec<&str> = payment_str.split('/').collect();

    let method = if !parts[0].is_empty() {
        Some(parts[0].to_string())
    } else {
        None
    };

    let date = if parts.len() > 1 {
        match parse_date(parts[1]) {
            Ok(parsed_date) => Some(parsed_date),
            Err(_) => None, // 예상하지 못한 형식일 경우 None
        }
    } else {
        None
    };

    Ok((method, date))
}

// get text from locator
async fn get_text_by_locator(client: &Client, locator: Locator<'_>) -> Option<String> {
    match client.find(locator).await.ok() {
        Some(element) => element.text().await.ok(),
        None => None,
    }
}

// get text from locator art index
async fn get_text_by_locator_at_index(
    client: &Client,
    locator: Locator<'_>,
    index: usize,
) -> Option<String> {
    match client.find_all(locator).await.ok() {
        Some(elements) => {
            if let Some(element) = elements.get(index) {
                return element.text().await.ok();
            }
            None
        }
        None => None,
    }
}

// get_and_parsing_data year
async fn extract_data_year(client: &Client, parent_id: &str) -> Result<KepcoData> {
    let claim_date_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payYm')]",
            parent_id
        )),
    )
    .await;

    let date_range_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_gigan')]",
            parent_id
        )),
    )
    .await;

    let usage_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_useKwh')]",
            parent_id
        )),
    )
    .await;

    let amount_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_monthPay')]",
            parent_id
        )),
    )
    .await;

    let paid_row = get_text_by_locator_at_index(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_pay')]",
            parent_id
        )),
        1,
    )
    .await;

    let unpaid_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payAmt')]",
            parent_id
        )),
    )
    .await;

    let payment_option_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payGubnNDay')]",
            parent_id
        )),
    )
    .await;

    let claim_date = claim_date_row.map(|date| parse_date(&date)).transpose()?;
    let (start_date, end_date) = date_range_row
        .map(|range| parse_date_range(&range))
        .transpose()?
        .unwrap_or((None, None));
    let usage = usage_row.map_or(Ok(0.0), |kwh| parse_use_kwh(&kwh))?;
    let amount = amount_row.map_or(Ok(0), |amount| parse_amount(&amount))?;
    let paid = paid_row.map_or(Ok(0), |paid| parse_amount(&paid))?;
    let unpaid = unpaid_row.map_or(Ok(0), |unpaid| parse_amount(&unpaid))?;
    let (payment_method, payment_date) =
        payment_option_row.map_or(Ok((None, None)), |s| parse_payment_method(&s))?;

    Ok(KepcoData {
        claim_date,
        start_date,
        end_date,
        usage,
        amount,
        paid,
        unpaid,
        payment_method,
        payment_date,
    })
}

// parse_data_from_parent_ids
async fn parse_data_from_parent_ids(
    client: &Arc<Client>,
    map: Arc<DashMap<String, ()>>,
) -> Result<Vec<KepcoData>> {
    let mut tasks = vec![];

    for entry in map.iter() {
        let id = entry.key().clone();
        let client = Arc::clone(client);
        let task = tokio::spawn(async move { extract_data_year(&client, &id).await });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    let mut data_vec = Vec::new();
    for result in results {
        match result {
            Ok(Ok(data)) => data_vec.push(data),
            Ok(Err(e)) => eprintln!("Failed to extract data: {}", e),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }

    Ok(data_vec)
}

// get_and_parsing_data monthly
async fn extract_data_month(client: &Client, parent_id: &str) -> Result<KepcoData> {
    let claim_date_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payYm')]",
            parent_id
        )),
    )
    .await;

    let usage_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_useKwh')]",
            parent_id
        )),
    )
    .await;

    let amount_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_monthPay')]",
            parent_id
        )),
    )
    .await;

    let paid_row = get_text_by_locator_at_index(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_pay')]",
            parent_id
        )),
        1,
    )
    .await;

    let unpaid_row = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payAmt')]",
            parent_id
        )),
    )
    .await;

    let payment_method = get_text_by_locator(
        client,
        Locator::XPath(&format!(
            "//*[@id='{}']//span[contains(@id, '_txt_payGubn')]",
            parent_id
        )),
    )
    .await;

    let claim_date = claim_date_row.map(|date| parse_date(&date)).transpose()?;
    let usage = usage_row.map_or(Ok(0.0), |kwh| parse_use_kwh(&kwh))?;
    let amount = amount_row.map_or(Ok(0), |amount| parse_amount(&amount))?;
    let paid = paid_row.map_or(Ok(0), |paid| parse_amount(&paid))?;
    let unpaid = unpaid_row.map_or(Ok(0), |unpaid| parse_amount(&unpaid))?;

    Ok(KepcoData {
        claim_date,
        start_date: None,
        end_date: None,
        usage,
        amount,
        paid,
        unpaid,
        payment_method,
        payment_date: None,
    })
}

// options 들의 결과값 parsing
async fn parsing_options_data(
    client: &Arc<Client>,
    select_locator: Locator<'_>,
    user_number: &str,
    option_index: &usize,
    chromedriver_process: &mut Child,
) -> Result<Vec<KepcoData>> {
    let mut kepco_data_vec: Vec<KepcoData> = Vec::new();

    // option 요소
    let options = client
        .find(select_locator)
        .await
        .context("Failed to find select element")?
        .find_all(Locator::Css("option"))
        .await
        .context("Failed to find options")?;

    // option_index to last index data parsing
    for i in *option_index..options.len() {
        // 옵션 선택
        options[i]
            .click()
            .await
            .context("Failed to select option")?;

        // 로딩 대기
        wait_for_element_hidden(
            client,
            Locator::Id("mf_wq_uuid_1_wq_processMsgComp"),
            chromedriver_process,
            Duration::from_secs(20),
        )
        .await?;

        // 고객 번호 입력
        enter_value_in_element(
            client,
            Locator::Id("mf_wfm_layout_inp_searchCustNo"),
            user_number,
        )
        .await
        .context("Failed to enter customer number")?;

        // 검색 버튼 클릭
        click_element(client, Locator::Id("mf_wfm_layout_btn_search")).await?;

        // data box 로드 대기
        wait_for_element(
            client,
            Locator::Id("mf_wfm_layout_ui_generator"),
            chromedriver_process,
        )
        .await?;

        // data parsing
        let data = extract_data_month(client, "mf_wfm_layout_ui_generator").await?;
        kepco_data_vec.push(data);
    }

    Ok(kepco_data_vec)
}
