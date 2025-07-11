use serde_derive::Deserialize;

pub const PP_URL: &str = "https://pp.kepco.co.kr";
pub const HOME_URL: &str = "https://pp.kepco.co.kr/rm/rm0101.do?menu_id=O010101";
pub const USER_INFO_URL: &str = "https://pp.kepco.co.kr/mb/mb0101.do?menu_id=O010601";
pub const USER_SELECT_CHARGE_URL: &str = "https://pp.kepco.co.kr/pf/pf0101_1.do?menu_id=O010501";

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
