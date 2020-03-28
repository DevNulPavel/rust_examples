
use crate::core::user_id::UserId;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: UserId,
    pub to: UserId,
    pub amount: i128,
}