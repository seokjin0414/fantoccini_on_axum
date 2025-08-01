use anyhow::{Result, anyhow};
use serde_derive::Serialize;
use tempfile::TempDir;

const BINARY_PATH: &str = "CHROME_BINARY_PATH";
const BINARY_PATH_TEST: &str = "CHROME_BINARY_PATH_TEST";
const HEADLESS: &str = "--headless";
const DISABLE_GPU: &str = "--disable-gpu";
const DISABLE_TRANSLATE: &str = "--disable-translate";
const DISABLE_BACKGROUND_TIMER_THROTTLING: &str = "--disable-background-timer-throttling";
const DISABLE_CLIENT_SIDE_PHISHING_DETECTION: &str = "--disable-client-side-phishing-detection";
const DISABLE_DEFAULT_APPS: &str = "--disable-default-apps";
const DISABLE_FEATURES_TRANSLATE_UI: &str = "--disable-features=TranslateUI";
const NO_SANDBOX: &str = "--no-sandbox";
const NO_FIRST_RUN: &str = "--no-first-run";
const MUTE_AUDIO: &str = "--mute-audio";
const WINDOW_SIZE: &str = "--window-size=774,857";

const COMMON_ARGS: &[&str] = &[
    HEADLESS,
    DISABLE_GPU,
    DISABLE_TRANSLATE,
    DISABLE_BACKGROUND_TIMER_THROTTLING,
    DISABLE_CLIENT_SIDE_PHISHING_DETECTION,
    DISABLE_DEFAULT_APPS,
    DISABLE_FEATURES_TRANSLATE_UI,
    NO_FIRST_RUN,
    MUTE_AUDIO,
    WINDOW_SIZE,
];

pub const LOCAL_URL: &str = "http://localhost:4450";

#[derive(Serialize)]
pub struct ChromeOptions {
    binary: String,
    args: Vec<String>,
    #[serde(skip)]
    pub _user_data_tempdir: Option<TempDir>,
}

impl ChromeOptions {
    fn with_binary(binary: String) -> Result<Self> {
        let tmp_dir = TempDir::new()
            .map_err(|e| anyhow!("Failed to create temp dir: {}", e))?;

        let mut args = COMMON_ARGS
            .iter()
            .map(|&s| s.to_string())
            .collect::<Vec<_>>();
        args.push(format!("--user-data-dir={}", tmp_dir.path().display()));

        Ok(ChromeOptions { binary, args, _user_data_tempdir: Some(tmp_dir) })
    }

    pub fn new(test: bool) -> Result<Self> {
        let mut key = BINARY_PATH_TEST;
        if test {
            key = BINARY_PATH
        }

        let binary_path =
            std::env::var(key).map_err(|e| anyhow!("Failed to get ENV BINARY_PATH: {}", e))?;

        Self::with_binary(binary_path)
    }
}
