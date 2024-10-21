use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct PpRequestBody {
    pub userId: String,
    pub userPw: String,
    pub userNum: String,
}

