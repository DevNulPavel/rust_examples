mod blockchain;
mod block;
mod chain;
mod user_id;

use url::Url;

pub use blockchain::Blockchain;
pub use block::Block;
pub use chain::Chain;
pub use user_id::UserId;

static SYSTEM_USER_ID: UserId = UserId(0);

/*
 * - Mistake in proof of work https://medium.com/@schubert.konstantin/isnt-there-a-msitake-with-your-proof-of-work-30cf9467f0a5
 * - Need to hash the last block when adding a new block
 */

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockIndex(pub u128);

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockProof(pub u128);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct BlockHash(pub String);

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: UserId,
    pub to: UserId,
    pub amount: i128,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(with = "url_serde")]
    address: Url,
}

//////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chain() {
        let mut bloxi = Blockchain::new();
        assert!(bloxi.chain().is_valid()); // all chains are initially valid
        for i in 1..5 {
            bloxi.add_transaction(Transaction {
                from: UserId(i),
                to: UserId(i + 1),
                amount: (i + 2) as i128,
            });
            bloxi.add_transaction(Transaction {
                from: UserId(i + 3),
                to: UserId(i + 4),
                amount: (i + 5) as i128,
            });
            bloxi.add_transaction(Transaction {
                from: UserId(i + 6),
                to: UserId(i + 7),
                amount: (i + 8) as i128,
            });
            bloxi.mine();
        }
        assert!(bloxi.chain().is_valid());
    }
}
