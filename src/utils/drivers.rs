use tokio::process::{Command, Child};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use fantoccini::{Client, ClientBuilder, Locator};
use fantoccini::elements::Element;
use fantoccini::wd::Capabilities;
use futures::TryFutureExt;
use tokio::sync::Mutex;
use tokio::time::timeout;
use crate::models::driver::chromes::ChromeOptions;

pub async fn start_chromedriver() -> Result<Child> {
    let path = std::env::var("CHROME_DRIVER_PATH")
        .map_err(|e| anyhow!("Failed to get CHROME_DRIVER_PATH: {:?}", e))?;

    let chromedriver_process = Command::new(path)
        .arg("--port=4450")
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("Failed to start ChromeDriver: {:?}", e))?;

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
    let chrome_options = serde_json::to_value(ChromeOptions::new(test))
        .map_err(|e| anyhow!("Failed to serialize ChromeOptions: {:?}", e))?;

    capabilities.insert("goog:chromeOptions".to_string(), chrome_options);
    Ok(capabilities)
}

pub async fn create_client(url: &str, test: bool) -> Result<Arc<Client>> {
    let client = ClientBuilder::native()
        .capabilities(create_capabilities(test)?)
        .connect(url)
        .await
        .map_err(|e| anyhow!("Failed to connect process: {:?}", e))?;

    Ok(Arc::new(client))
}

pub async fn go_to_url(client: &Client, url: &str) -> Result<()> {
    client.goto(&format!("{}", url)).await
        .map_err(|e| anyhow!("Failed to client goto URL({})\n {:?}", url, e))?;

    Ok(())
}

pub async fn find_element(client: &Client, locator: Locator<'_>) -> Result<Element> {
    let element= client.find(locator).await
        .map_err(|e| anyhow!("Failed to find element: {:?}\n {:?}", locator, e))?;

    Ok(element)
}

pub async fn wait_element(client: &Client, locator: Locator<'_>) -> Result<Element> {
    let element = client.wait().for_element(locator)
        .await
        .map_err(|e| anyhow!("Failed to wait element: {:?}\n {}", locator, e))?;

    Ok(element)
}

pub async fn click_element(client: &Client, locator: Locator<'_>) -> Result<()> {
    let element= find_element(client, locator).await?;

    element.click().await
        .map_err(|e| anyhow!("Failed to click the element: {:?}\n {:?}", locator, e))?;

    println!("clicked successfully: {:?}", locator);
    Ok(())
}

pub async fn enter_value_in_element(client: &Client, locator: Locator<'_>, text: &str) -> Result<()> {
    let element= find_element(client, locator).await?;

    element.send_keys(text).await
       .map_err(|e| anyhow!("Failed to send keys: {:?}\n {:?}", locator, e))?;

    println!("entered successfully: {:?}", locator);
    Ok(())
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
                Ok(_) => {
                    anyhow!("Element is not hidden, retrying...");
                }
                Err(e) => {
                    anyhow!("Failed to get style attribute: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
        .await
        .map_err(|e| anyhow!("Failed to wait the element within the given duration: {:?}", e))?;

    Ok(())
}