use serde_derive::{Deserialize, Serialize};
use anyhow::{anyhow, Result};

#[derive(Serialize)]
struct ChromeOptions {
    binary: Option<String>,
    args: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Capabilities {
    #[serde(rename = "goog:chromeOptions")]
    chrome_options: ChromeOptions,
}

impl Capabilities {
    fn new() -> Result<()> {
        let capabilities = Capabilities {
            chrome_options: ChromeOptions {
                binary: None,
                args: vec!["--headless".to_string(), "--disable-gpu".to_string()],
            },
        };

        let capabilities_value = serde_json::to_value(&capabilities)
            .map_err(|e| anyhow!(e))?;



        let capabilities_map = capabilities_value.as_object();

    }
}