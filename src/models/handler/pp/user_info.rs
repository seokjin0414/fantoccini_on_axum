use serde_derive::Serialize;

pub const CONTRACT_TYPE: &str = "contract_type";
pub const PURPOSE: &str = "purpose";
pub const PURPOSE_ID: i16 = 1;
pub const CONTRACT_TYPE_ID: i16 = 25;

pub fn purpose_vec() -> Vec<(&'static str, i16)> {
    vec![
        ("-", 1),
        ("고압", 4),
        ("고압선택1", 6),
        ("고압선택2", 7),
        ("고압선택3", 8),
        ("고압선택4", 9),
        ("고압A", 10),
        ("고압A선택1", 11),
        ("고압A선택2", 12),
        ("고압A선택3", 13),
        ("고압B", 14),
        ("고압B선택1", 15),
        ("고압B선택2", 16),
        ("고압B선택3", 17),
        ("고압C", 18),
        ("고압C선택1", 19),
        ("고압C선택2", 20),
        ("고압C선택3", 21),
        ("저압", 23),
        ("저압선택1", 24),
        ("저압선택2", 25),
        ("저압선택3", 26),
        ("저압선택4", 27),
    ]
}

pub fn contract_vec() -> Vec<(&'static str, i16)> {
    vec![
        ("가로등(갑)", 1),
        ("가로등(을)", 26),
        ("교육용(갑)", 2),
        ("교육용(을)", 3),
        ("농사용(갑)", 4),
        ("농 사용(을)", 27),
        ("산업용(갑)I", 6),
        ("산업용(갑)II", 7),
        ("산업용(을)", 8),
        ("심야전력(갑)", 9),
        ("심야전력(을)I", 10),
        ("심야전력(을)II", 11),
        ("일반용 보완", 12),
        ("일반용(갑)I", 13),
        ("일반용(갑)II", 14),
        ("일반용(을)", 15),
        ("주택용", 20),
        ("-", 25),
    ]
}

#[derive(Serialize, Debug)]
pub struct UserInfo {
    pub user_number: String,
    pub contract_type_id: i16,
    pub purpose_id: i16,
    pub contract_power: f64,
    pub inspection_day: i16,
    pub instrument_number: String,
}
