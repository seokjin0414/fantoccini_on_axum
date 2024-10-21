use chrono::NaiveDate;
use serde_derive::Serialize;

#[derive(Serialize, Debug)]
pub struct UserInfo {
    pub user_number: String,
    pub contract_type: String,
    pub contract_power: String,
    pub inspection_day: String,
    pub instrument_number: String,
}