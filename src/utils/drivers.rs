use tokio::process::{Command, Child};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use fantoccini::{Client, ClientBuilder, Locator};
use fantoccini::elements::Element;
use fantoccini::wd::Capabilities;
use futures::TryFutureExt;
use tokio::sync::Mutex;
use tokio::time::timeout;
use crate::models::driver::chromes::ChromeOptions;
use crate::models::error::response_errors_def::ErrorResponseCode;

pub async fn start_chromedriver() -> Result<Child> {
    let path = std::env::var("CHROME_DRIVER_PATH")
        .map_err(|e| anyhow!("Failed to get CHROME_DRIVER_PATH: {:?}", e))?;

    let chromedriver_process = Command::new(path)
        .arg("--port=4450")
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            eprintln!("Failed to start ChromeDriver: {:?}", e);
            anyhow!("Failed to start ChromeDriver: {:?}", e)
        })?;

    println!("ChromeDriver started on port 4450");
    Ok(chromedriver_process)
}

pub async fn shutdown_chromedriver(chromedriver_process: Arc<Mutex<Child>>) {
    let mut process = chromedriver_process.lock().await;

    if let Ok(Some(_status)) = process.try_wait() {
        println!("ChromeDriver process already terminated");
    } else {
        if let Err(e) = process.kill().await {
            eprintln!("Failed to kill ChromeDriver: {:?}", e);
        } else {
            println!("ChromeDriver process terminated");
        }
    }
}

pub fn create_capabilities(test: bool) -> Result<Capabilities> {
    let mut capabilities = Capabilities::new();
    let chrome_options = serde_json::to_value(ChromeOptions::new(test)?)
        .map_err(|e| {
            eprintln!("Failed to connect process: {:?}", e);
            anyhow!("Failed to serialize ChromeOptions: {:?}", e)
        })?;

    capabilities.insert("goog:chromeOptions".to_string(), chrome_options);
    Ok(capabilities)
}

pub async fn create_client(url: &str, test: bool) -> Result<Arc<Client>> {
    let client = ClientBuilder::native()
        .capabilities(create_capabilities(test)?)
        .connect(url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect process: {:?}", e);
            anyhow!("Failed to connect process: {:?}", e)
        })?;

    Ok(Arc::new(client))
}

pub async fn go_to_url(client: &Client, url: &str) -> Result<()> {
    client.goto(url).await
        .map_err(|e| {
            eprintln!("Failed to client goto URL({})\n {:?}", url, e);
            anyhow!("Failed to client goto URL({})\n {:?}", url, e)
        })?;

    Ok(())
}

pub async fn find_element(client: &Client, locator: Locator<'_>) -> Result<Element> {
    let element= client.find(locator).await
        .map_err(|e| {
            eprintln!("Failed to find element: {:?}\n {:?}", locator, e);
            anyhow!("Failed to find element: {:?}\n {:?}", locator, e)
        })?;

    Ok(element)
}

pub async fn wait_element(client: &Client, locator: Locator<'_>) -> Result<Element> {
    let element = client.wait().for_element(locator)
        .await
        .map_err(|e| {
            eprintln!("Failed to wait element: {:?}\n {}", locator, e);
            anyhow!("Failed to wait element: {:?}\n {}", locator, e)
        })?;

    Ok(element)
}

pub async fn click_element(client: &Client, locator: Locator<'_>) -> Result<()> {
    let element= find_element(client, locator).await?;

    element.click().await
        .map_err(|e| {
            eprintln!("Failed to click the element: {:?}\n {:?}", locator, e);
            anyhow!("Failed to click the element: {:?}\n {:?}", locator, e)
        })?;

    Ok(())
}

pub async fn enter_value_in_element(client: &Client, locator: Locator<'_>, text: &str) -> Result<()> {
    let element= find_element(client, locator).await?;

    element.send_keys(text).await
       .map_err(|e| {
           eprintln!("Failed to send keys to element: {:?}\n {:?}", locator, e);
           anyhow!("Failed to send keys to element: {:?}\n {:?}", locator, e)
       })?;

    Ok(())
}

pub async fn attr_element(client: &Client, locator: Locator<'_>, attr: &str) -> Result<Option<String>> {
    let element = find_element(client, locator).await?;

    let attr = element.attr(attr)
        .await
        .map_err(|e| {
            eprintln!("Failed to get attr from element: {:?}", e);
            anyhow!("Failed to get attr from element: {:?}", e)
        })?;

    Ok(attr)
}

pub async fn text_element(client: &Client, locator: Locator<'_>) -> Result<String> {
    let element = find_element(client, locator).await?;

    let text = element.text()
        .await
        .map_err(|e| {
            eprintln!("Failed to get text from element: {:?}", e);
            anyhow!("Failed to get text from element: {:?}", e)
        })?;

    Ok(text)
}

pub async fn wait_for_element_display_none(
    client: &Client,
    locator: Locator<'_>,
    duration: Duration,
) -> Result<()> {
    let element = wait_element(client, locator).await?;

    timeout(duration, async {
        loop {
            match element.attr("style").await {
                Ok(Some(style)) if style.contains("display: none") => {
                    println!("Element is hidden (style=\"display: none\")");
                    break;
                }
                _ => {
                    println!("Element is not hidden, retrying...");
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
            }
        }
    })
        .await
        .map_err(|e| {
            eprintln!("Failed to wait the element within the given duration: {:?}", e);
            anyhow!("Failed to wait the element within the given duration: {:?}", e)
        })?;

    Ok(())
}

pub async fn clean_client(client: &Client) -> Result<()> {
    client.delete_all_cookies()
        .await
        .map_err(|e| {
            eprintln!("Failed to delete client cookies: {:?}", e);
            anyhow!("Failed to delete client cookies: {:?}", e)
        })?;

    Ok(())
}