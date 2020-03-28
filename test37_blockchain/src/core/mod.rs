mod blockchain;
mod block;
mod chain;
mod user_id;
mod transaction;
mod proof;
mod node;
mod hash;
mod index;

#[cfg(test)]
mod test;

pub use blockchain::Blockchain;
pub use block::Block;
pub use chain::Chain;
pub use user_id::UserId;
pub use transaction::Transaction;
pub use proof::BlockProof;
pub use node::Node;
pub use index::BlockIndex;

static SYSTEM_USER_ID: UserId = UserId(0);

/*
 * - Mistake in proof of work https://medium.com/@schubert.konstantin/isnt-there-a-msitake-with-your-proof-of-work-30cf9467f0a5
 * - Need to hash the last block when adding a new block
 */


//////////////////////////////////////////////////////////////////////////////////////////
