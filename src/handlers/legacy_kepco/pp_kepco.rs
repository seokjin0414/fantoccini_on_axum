use anyhow::{Context, Result};
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDate;
use dashmap::DashMap;
use fantoccini::{elements::Element, Client, ClientBuilder, Locator};
use serde_json::{json, Map, Value};
use std::{
    collections::HashSet,
    process::{Child, Command},
    sync::Arc,
};
use tokio::time::{timeout, Duration};

use crate::models::handler::{
    legacy_kepco::pp_models::{
        MetaResponseData, PpAllPeriodsPaidData, PpAllPeriodsPaidDataResponse,
    },
    pp::commons::PpRequestBody
};

// 파워 플레너 모든기간 요금 조회 고객번호 기준
pub async fn get_pp_all_periods_paid_data_handler(
    Json(params): Json<PpRequestBody>,
) -> impl IntoResponse {
    let url = "http://localhost:4445";
    let target_url = "https://pp.kepco.co.kr";
    let user_id = &params.userId;
    let user_pw = &params.userPw;
    let user_number = &params.userNum;

    // driver path
    let chromedriver_path = "/opt/homebrew/bin/chromedriver";
    // let chromedriver_path = "src/driver/chromedriver";
    // let chrome_binary_path = "/usr/bin/google-chrome";

    // driver 실행
    let mut chromedriver_process = match Command::new(chromedriver_path)
        .arg("--port=4445")
    // .arg(format!("--binary={}", chrome_binary_path))
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
            // "binary": "/usr/bin/google-chrome",
            "args": ["--headless", "--disable-gpu"]
        }
    })) {
        Ok(result) => result,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not acquire connection from pool!",
            ).into_response()
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
                tokio::time::sleep(Duration::from_millis(300)).await;
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
    match client_arc.goto(&format!("{}/intro.do", target_url)).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to navigate goto!: {:?}", e),
            )
                .into_response();
        }
    };

    // 공지 팝업 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("notice_auto_popup"),
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
    //공지 팝업 비활성화
    match click_element(
        &client_arc,
        Locator::XPath("/html/body/div[2]/div[3]/label"),
    )
    .await
    {
        Ok(_) => {}
        Err(_) => {}
    };

    // id 입력 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("RSA_USER_ID"),
        &mut chromedriver_process,
    )
    .await
    {
        Ok(_) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to wait_for_element!",
            )
                .into_response()
        }
    };
    // id 입력
    match enter_value_in_element(&client_arc, Locator::Id("RSA_USER_ID"), user_id).await {
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
    match enter_value_in_element(&client_arc, Locator::Id("RSA_USER_PWD"), user_pw).await {
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
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/form/fieldset/input[1]"),
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

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    // user_num selector 클릭
    match click_element(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[1]/div/div/a[2]"),
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
    // user_num 클릭
    match click_element(
        &client_arc,
        Locator::XPath(
            format!(
                "/html/body/div[1]/div[1]/div/div/ul/li[1]/a[text()='{}']",
                user_number,
            )
            .as_str(),
        ),
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

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    // get 월별 청구 요금 url
    let monthly_claim_href = get_href_by_locator(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/ul[4]/li[5]/a"),
    )
    .await
    .unwrap_or_else(|| "".to_string());

    let claim_url = format!("{}{}", target_url, monthly_claim_href);

    // 월별 청구 요금 이동
    match client_arc.goto(&claim_url).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed go to monthly_claim_href: {:?}", e),
            )
                .into_response()
        }
    };

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    // data from table -> vec
    let mut data_vec = match parse_data_from_table(&client_arc, "//*[@id='grid']/tbody").await {
        Ok(vec) => vec,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not Parse data_from_table!: {:?}", e),
            )
                .into_response()
        }
    };

    // select locator
    let select_locator = Locator::Id("year");

    // 1year over data parsing
    let mut additional_data_vec = match parsing_options_data(
        &client_arc,
        select_locator,
        &1,
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

    // 중복 제거
    let mut unique_dates = HashSet::new();
    data_vec.retain(|entry| unique_dates.insert(entry.claim_date));

    // 정렬
    data_vec.sort_by(|a, b| b.claim_date.cmp(&a.claim_date));

    let response: PpAllPeriodsPaidDataResponse = PpAllPeriodsPaidDataResponse {
        data: data_vec,
        meta: MetaResponseData {},
    };

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

    (StatusCode::OK, Json(response)).into_response()
}

// latest 3 data
pub async fn get_latest_3_pp_paid_data_handler(
    Json(params): Json<PpRequestBody>,
) -> impl IntoResponse {
    let url = "http://localhost:4450";
    let target_url = "https://pp.kepco.co.kr";
    let user_id = &params.userId;
    let user_pw = &params.userPw;
    let user_number = &params.userNum;

    // driver path
    let chromedriver_path = "/opt/homebrew/bin/chromedriver";
    // let chromedriver_path = "src/driver/chromedriver";
    // let chrome_binary_path = "/usr/bin/google-chrome";

    // driver 실행
    let mut chromedriver_process = match Command::new(chromedriver_path)
        .arg("--port=4450")
        // .arg(format!("--binary={}", chrome_binary_path))
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
            // "binary": "/usr/bin/google-chrome",
            "args": ["--headless", "--disable-gpu"]
        }
    })) {
        Ok(result) => result,
        Err(_) => {
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
                tokio::time::sleep(Duration::from_millis(300)).await;
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
    match client_arc.goto(&format!("{}/intro.do", target_url)).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to navigate goto!: {:?}", e),
            )
                .into_response();
        }
    };

    // 공지 팝업 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("notice_auto_popup"),
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
    //공지 팝업 비활성화
    match click_element(
        &client_arc,
        Locator::XPath("/html/body/div[2]/div[3]/label"),
    )
        .await
    {
        Ok(_) => {}
        Err(_) => {}
    };

    // id 입력 로드 대기
    match wait_for_element(
        &client_arc,
        Locator::Id("RSA_USER_ID"),
        &mut chromedriver_process,
    )
        .await
    {
        Ok(_) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to wait_for_element!",
            )
                .into_response()
        }
    };
    // id 입력
    match enter_value_in_element(&client_arc, Locator::Id("RSA_USER_ID"), user_id).await {
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
    match enter_value_in_element(&client_arc, Locator::Id("RSA_USER_PWD"), user_pw).await {
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
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/form/fieldset/input[1]"),
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

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    // user_num selector 클릭
    match click_element(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[1]/div/div/a[2]"),
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
    // user_num 클릭
    match click_element(
        &client_arc,
        Locator::XPath(
            format!(
                "/html/body/div[1]/div[1]/div/div/ul/li[1]/a[text()='{}']",
                user_number,
            )
                .as_str(),
        ),
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

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    // get 월별 청구 요금 url
    let monthly_claim_href = get_href_by_locator(
        &client_arc,
        Locator::XPath("/html/body/div[1]/div[2]/div[1]/ul[4]/li[5]/a"),
    )
        .await
        .unwrap_or_else(|| "".to_string());

    let claim_url = format!("{}{}", target_url, monthly_claim_href);

    // 월별 청구 요금 이동
    match client_arc.goto(&claim_url).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed go to monthly_claim_href: {:?}", e),
            )
                .into_response()
        }
    };

    // 로딩 대기
    match wait_for_element_display_none(
        &client_arc,
        Locator::Id("backgroundLayer"),
        &mut chromedriver_process,
        Duration::from_secs(20),
    )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to wait_for_element_display_none!: {:?}", e),
            )
                .into_response()
        }
    };

    todo!();
    // parse_data_from_table -> get latest 3 data

    // data from table -> vec
    let mut data_vec = match parse_data_from_table(&client_arc, "//*[@id='grid']/tbody").await {
        Ok(vec) => vec,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not Parse data_from_table!: {:?}", e),
            )
                .into_response()
        }
    };

    // 정렬
    data_vec.sort_by(|a, b| b.claim_date.cmp(&a.claim_date));

    let response: PpAllPeriodsPaidDataResponse = PpAllPeriodsPaidDataResponse {
        data: data_vec,
        meta: MetaResponseData {},
    };

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

    (StatusCode::OK, Json(response)).into_response()
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
async fn wait_for_element_display_none(
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
            match element.attr("style").await {
                Ok(Some(style)) if style.contains("display: none") => {
                    println!("Element is hidden (style=\"display: none\")");
                    break;
                }
                Ok(_) => {
                    eprintln!("Element is not hidden, retrying...");
                }
                Err(e) => {
                    eprintln!("Failed to get style attribute: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
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

// 자식 요소들의 ID -> DashMap
async fn get_children_ids_to_map(
    client: &Client,
    parent_xpath: &str,
) -> Result<Arc<DashMap<String, ()>>> {
    let script = format!(
        r#"
        let parent = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
        if (parent === null) {{
            throw new Error('Parent element not found');
        }}
        let children = parent.querySelectorAll('tr');
        let ids = [];
        for (let i = 0; i < children.length; i++) {{
            ids.push(children[i].id);
        }}
        return ids;
        "#,
        parent_xpath
    );

    let result = client
        .execute(&script, vec![])
        .await
        .context("Failed to execute script to get children IDs")?;

    let ids: Vec<String> = result
        .as_array()
        .context("Expected an array from the script result")?
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let map = Arc::new(DashMap::new());
    for id in ids {
        map.insert(id, ());
    }

    Ok(map)
}

// get text from locator
async fn get_text_by_locator(client: &Client, locator: Locator<'_>) -> Option<String> {
    match client.find(locator).await.ok() {
        Some(element) => element.text().await.ok(),
        None => None,
    }
}

// get href from locator
async fn get_href_by_locator(client: &Client, locator: Locator<'_>) -> Option<String> {
    match client.find(locator).await.ok() {
        Some(element) => element.attr("href").await.ok().flatten(),
        None => None,
    }
}

// parsing 청구 기간
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    // 일자를 1로 설정
    let date_with_day = format!("{} 01일", date_str);
    NaiveDate::parse_from_str(&date_with_day, "%Y년 %m월 %d일").context("Failed to parse date")
}

// parsing 사용량
fn parse_use_kwh(kwh_str: &str) -> Result<f64> {
    let cleaned_str = kwh_str.replace(",", "").replace("kWh", "");
    cleaned_str
        .parse::<f64>()
        .context("Failed to parse use kWh")
}

// parsing 요금
fn parse_paid(amount_str: &str) -> Result<i64> {
    let amount_part = amount_str.split('원').next().unwrap_or(amount_str);

    let amount = amount_part.replace(",", "").replace(".", "");
    amount.parse::<i64>().context("Failed to parse amount")
}

// get_and_parsing_data year
async fn extract_data_year(client: &Client, parent_id: &str) -> Result<PpAllPeriodsPaidData> {
    let claim_date_row = get_text_by_locator(
        client,
        Locator::XPath(&format!("//*[@id='{}']/td[1]/a/span", parent_id)),
    )
    .await;

    let usage_row = get_text_by_locator(
        client,
        Locator::XPath(&format!("//*[@id='{}']/td[4]", parent_id)),
    )
    .await;

    let paid_row = get_text_by_locator(
        client,
        Locator::XPath(&format!("//*[@id='{}']/td[8]", parent_id)),
    )
    .await;

    let claim_date = claim_date_row.map_or(Ok(Default::default()), |date| parse_date(&date))?;
    let usage = usage_row.map_or(Ok(0.0), |kwh| parse_use_kwh(&kwh))?;
    let paid = paid_row.map_or(Ok(0), |paid| parse_paid(&paid))?;

    Ok(PpAllPeriodsPaidData {
        claim_date,
        usage,
        paid,
    })
}

// parse_data_from_parent_ids
async fn parse_data_from_table(
    client: &Arc<Client>,
    parent_xpath: &str,
) -> Result<Vec<PpAllPeriodsPaidData>> {
    let mut tasks = vec![];

    let map = get_children_ids_to_map(&client, parent_xpath).await?;

    for entry in map.iter() {
        let id = entry.key().clone();
        let client = Arc::clone(&client);
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

// options 들의 결과값 parsing
async fn parsing_options_data(
    client: &Arc<Client>,
    select_locator: Locator<'_>,
    option_index: &usize,
    chromedriver_process: &mut Child,
) -> Result<Vec<PpAllPeriodsPaidData>> {
    // option 요소
    let options = client
        .find(select_locator)
        .await
        .context("Failed to find select element")?
        .find_all(Locator::Css("option"))
        .await
        .context("Failed to find options")?;

    let mut vec: Vec<PpAllPeriodsPaidData> = Vec::with_capacity(options.len() * 12);

    // option_index to last index data parsing
    for i in *option_index..options.len() {
        // 옵션 선택
        options[i]
            .click()
            .await
            .context("Failed to select option")?;

        // 조회 버튼 클릭
        click_element(&client, Locator::XPath("//*[@id='txt']/div[2]/p/span[1]/a")).await?;

        // 로딩 대기
        wait_for_element_display_none(
            &client,
            Locator::Id("backgroundLayer"),
            chromedriver_process,
            Duration::from_secs(10),
        )
        .await?;

        // data parsing
        let mut data = parse_data_from_table(&client, "//*[@id='grid']/tbody").await?;
        vec.append(&mut data);
    }

    Ok(vec)
}
