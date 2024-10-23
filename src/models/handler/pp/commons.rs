use serde_derive::Deserialize;

pub const HOME_URL: &str = "https://pp.kepco.co.kr";

#[derive(Deserialize, Debug, Clone)]
pub struct PpRequestBody {
    pub userId: String,
    pub userPw: String,
    pub userNum: String,
    pub testMode: Option<bool>,
}

impl PpRequestBody {
    pub fn test_state(&self) -> bool {
        self.testMode.unwrap_or(false)
    }
}

