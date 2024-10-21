use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PpRequestBody {
    pub userId: String,
    pub userPw: String,
    pub userNum: String,
}

