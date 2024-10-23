use serde_derive::{Deserialize, Serialize};

const BINARY_PATH: &str = "/usr/bin/chrome";
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
    binary: Option<String>,
    args: Vec<String>,
}

impl ChromeOptions {
    fn with_binary(binary: Option<String>) -> Self {
        ChromeOptions {
            binary,
            args: COMMON_ARGS.iter().map(|&s| s.to_string()).collect(),
        }
    }

    pub fn new(test: bool) -> Self {
        let binary_path = if test {
            None
        } else {
            Some(BINARY_PATH.to_string())
        };

        Self::with_binary(binary_path)
    }
}